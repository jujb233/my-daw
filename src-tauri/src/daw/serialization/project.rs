//! 项目序列化模块
//!
//! 该模块负责把运行时 `AppState` 序列化到磁盘并在加载时还原，具体包括：
//! - 将项目元数据写入 `project.lua`（Lua 可读脚本，用于手动查看或重新加载）
//! - 将大量的 MIDI note 存入 SQLite 数据库 `data.db`（减小 Lua 文件体积）
//! - 保存插件实例的状态到 SQLite 中（`plugins` 表）
//! - 将项目里引用的插件（builtin/local/clap）或其 manifest/资源一并复制到 `project/plugins`，以保证项目在别的机器上可复现
//!
//! 文件中提供的主要 API 为 `ProjectManager::save_project` 与 `ProjectManager::load_project`。
use crate::audio::plugins::manager::PluginSource;
use crate::daw::model::{ArrangementTrack, Clip};
use crate::daw::serialization::schema::*;
use crate::daw::state::AppState;
use anyhow::Result;
use mlua::{Lua, Table};
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};

/// ProjectManager: 提供将 `AppState` 序列化为磁盘项目、以及从磁盘项目加载回 `ProjectSchema` 的静态方法。
///
/// 该类型仅包含静态方法（全部为 `pub fn`），作为一个命名空间使用，因此定义为空结构体。
pub struct ProjectManager;

impl ProjectManager {
        /// 将 `AppState` 序列化并保存到 `project_path`。
        ///
        /// 工作流程（概览）：
        /// 1. 创建项目目录并打开/初始化 SQLite `data.db`。
        /// 2. 将插件与它们的状态写入 `plugins` 表（状态为二进制 blob）
        /// 3. 将所有 MIDI note 写入 `notes` 表（按 clip 分组，减少 Lua 文件体积）
        /// 4. 复制项目使用的插件资源到 `plugins/`（`official`、`local`、`clap` 等子目录）
        /// 5. 复制引用的音频文件到 `assets/` 并生成 `project.lua` 导出脚本
        ///
        /// 注意：该函数会在必要时对各种资源文件进行 I/O 复制，并在遇到问题时打印错误而不致命中止。
        pub fn save_project(state: &AppState, project_path: &Path) -> Result<()> {
                if !project_path.exists() {
                        fs::create_dir_all(project_path)?;
                }

                let db_path = project_path.join("data.db");
                let mut conn = Connection::open(&db_path)?;

                // 初始化并确保数据库表存在：`plugins` 与 `notes`
                conn.execute(
                        "CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT,
                state BLOB
            )",
                        [],
                )?;

                conn.execute(
                        "CREATE TABLE IF NOT EXISTS notes (
                clip_id TEXT,
                note_index INTEGER,
                note INTEGER,
                start REAL,
                duration REAL,
                velocity REAL,
                PRIMARY KEY (clip_id, note_index)
            )",
                        [],
                )?;

                // 抓取并锁定运行时状态（多个 Mutex）：
                // - `arrangement_tracks`/`clips`/`mixer_tracks`/`active_plugins`：读入后用于生成 Lua 脚本与复制资源
                // - `plugin_instances`：从实例中获取二进制状态 blob
                let tracks = state.arrangement_tracks.lock().unwrap();
                let clips = state.clips.lock().unwrap();
                let mixer_tracks = state.mixer_tracks.lock().unwrap();
                let plugins = state.active_plugins.lock().unwrap();

                // 将插件实例状态保存到 `plugins` 表
                // 使用事务确保写入一致性（批量插入/替换）
                let instances = state.plugin_instances.lock().unwrap();
                let tx = conn.transaction()?;
                for plugin in plugins.iter() {
                        let state_blob = if let Some(instance) = instances.get(&plugin.id) {
                                if let Ok(inst) = instance.lock() {
                                        inst.get_state()
                                } else {
                                        Vec::new()
                                }
                        } else {
                                Vec::new()
                        };

                        tx.execute(
                                "INSERT OR REPLACE INTO plugins (id, name, state) VALUES (?1, ?2, ?3)",
                                params![plugin.id, plugin.name, state_blob],
                        )?;
                }

                // 写入所有 MIDI note 到 `notes` 表（按 clip 把 notes 持久化）
                for clip in clips.iter() {
                        for (idx, note) in clip.notes.iter().enumerate() {
                                tx.execute(
                    "INSERT OR REPLACE INTO notes (clip_id, note_index, note, start, duration, velocity) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![clip.id, idx, note.note, note.start.time, note.duration.seconds, note.velocity],
                )?;
                        }
                }
                tx.commit()?;

                // 复制项目中使用的插件资源到 `project/plugins`（按类型分类：official/local/clap）
                let plugin_mgr = state.plugin_manager.lock().unwrap();
                let plugins_dir = project_path.join("plugins");
                if !plugins_dir.exists() {
                        fs::create_dir_all(&plugins_dir).ok();
                }

                for plugin in plugins.iter() {
                        match plugin_mgr.get_plugin_source(&plugin.id) {
                                PluginSource::Local(lib_path) => {
                                        // 向上查找插件 manifest 文件（从库文件路径向上）
                                        let mut folder = lib_path.clone();
                                        let mut found_manifest = false;
                                        while folder.parent().is_some() {
                                                if folder.join("manifest.lua").exists() {
                                                        found_manifest = true;
                                                        break;
                                                }
                                                if !folder.pop() {
                                                        break;
                                                }
                                        }

                                        if found_manifest {
                                                // 检查 manifest 的 `copy_on_project_save` 标志：
                                                // - true/缺省：拷贝整个插件文件夹到 `plugins/official`，并递归处理子 plugin
                                                // - false：跳过复制（项目中仍然可以引用该插件，但不会把资源复制到项目中）
                                                let manifest_path = folder.join("manifest.lua");
                                                let mut should_copy = true;
                                                if manifest_path.exists() {
                                                        if let Ok(content) = fs::read_to_string(&manifest_path) {
                                                                if let Ok(tbl) =
                                                                        Lua::new().load(&content).eval::<Table>()
                                                                {
                                                                        if let Ok(flag) =
                                                                                tbl.get::<bool>("copy_on_project_save")
                                                                        {
                                                                                should_copy = flag;
                                                                        }
                                                                }
                                                        }
                                                }

                                                if should_copy {
                                                        // 复制插件目录到 `plugins/official/<folder_name>`，保留原始文件结构
                                                        let folder_name = folder
                                                                .file_name()
                                                                .map(|s| s.to_string_lossy().to_string())
                                                                .unwrap_or(plugin.id.clone());
                                                        let dest = plugins_dir.join("official").join(folder_name);
                                                        if let Err(e) = Self::copy_dir_all(&folder, &dest) {
                                                                println!("Failed to copy plugin folder: {}", e);
                                                        }
                                                        // 如果 manifest 声明了 children，则递归复制这些子插件（子插件自己可通过 copy_on_project_save 控制）
                                                        if let Err(e) =
                                                                Self::copy_children_from_manifest(&folder, &plugins_dir)
                                                        {
                                                                println!("Failed to copy child plugins: {}", e);
                                                        }
                                                } else {
                                                        // manifest 指定不复制，跳过（保留 plugin 的引用而不拷贝文件）
                                                }
                                        } else {
                                                // 无 manifest：将库文件复制到 `plugins/local/<id>` 并生成最小 manifest 包含 id/name/backend 信息
                                                let dest_dir = plugins_dir.join("local").join(&plugin.id);
                                                if let Err(e) = fs::create_dir_all(&dest_dir) {
                                                        println!("Failed to create plugin dest dir: {}", e);
                                                }
                                                if let Some(fname) = lib_path.file_name() {
                                                        let _ = fs::copy(&lib_path, dest_dir.join(fname));
                                                        let manifest = format!(
                                                                "return {{ id = \"{}\", name = \"{}\", backend = {{ type = \"local\", path = \"{}\" }} }}\n",
                                                                plugin.id,
                                                                plugin.name,
                                                                fname.to_string_lossy()
                                                        );
                                                        let _ = fs::write(dest_dir.join("manifest.lua"), manifest);
                                                }
                                        }
                                }
                                PluginSource::Clap(p) => {
                                        // CLAP 插件可能是目录（包含插件结构）或文件（单体二进制），两者都支持复制到 `plugins/clap`。
                                        // 复制 clap 插件到 `plugins/clap`（目录或文件）
                                        let folder_name = p
                                                .file_name()
                                                .map(|s| s.to_string_lossy().to_string())
                                                .unwrap_or(plugin.id.clone());
                                        let dest = plugins_dir.join("clap").join(folder_name);
                                        if p.is_dir() {
                                                if let Err(e) = Self::copy_dir_all(&p, &dest) {
                                                        println!("Failed to copy clap plugin: {}", e);
                                                }
                                        } else {
                                                if let Some(parent) = dest.parent() {
                                                        let _ = fs::create_dir_all(parent);
                                                }
                                                let _ = fs::copy(&p, &dest);
                                        }
                                }
                                PluginSource::Builtin(module_opt) => {
                                        // builtin 插件没有实际二进制文件需要复制。为了让项目可复现，生成一个最小 manifest 并写入 `plugins/official/<id>/manifest.lua`。
                                        // 为 builtin 插件生成最小 manifest，便于项目可复现
                                        let dest_folder = plugins_dir.join("official").join(&plugin.id);
                                        if let Err(e) = fs::create_dir_all(&dest_folder) {
                                                println!("Failed to create builtin plugin folder: {}", e);
                                        }
                                        let module_field = if let Some(m) = module_opt {
                                                format!("module = \"{}\"", m)
                                        } else {
                                                String::new()
                                        };
                                        let manifest = format!(
                                                "return {{ id = \"{}\", name = \"{}\", backend = {{ type = \"builtin\", {} }} }}\n",
                                                plugin.id, plugin.name, module_field
                                        );
                                        let _ = fs::write(dest_folder.join("manifest.lua"), manifest);
                                        // builtin 插件通常不需要复制二进制文件
                                }
                                PluginSource::Unknown => {
                                        // nothing to copy
                                }
                        }
                }

                // 生成并写入 `project.lua`（用于查看或重新加载）
                let lua_script = Self::generate_lua_script(&tracks, &clips, &mixer_tracks, &plugins, project_path);
                fs::write(project_path.join("project.lua"), lua_script)?;

                Ok(())
        }

        /// 为项目生成 Lua 导出脚本（手工查看/加载友好）
        /// 将 tracks/clips/mixer/plugins 等内容序列化为一个人类可读的 Lua 脚本字符串。
        ///
        /// 这个导出脚本主要用于：
        /// - 在编辑器外部查看项目结构
        /// - 快速将项目载入到 Lua 环境以进行解析/调试
        fn generate_lua_script(
                tracks: &Vec<ArrangementTrack>,
                clips: &Vec<Clip>,
                mixer_tracks: &Vec<crate::daw::state::MixerTrackData>,
                plugins: &Vec<crate::daw::state::PluginInstanceData>,
                project_path: &Path,
        ) -> String {
                let mut script = String::new();

                script.push_str("-- MyDAW Project File\n");
                script.push_str("-- Generated by MyDAW Serializer\n\n");

                script.push_str("project {\n");
                script.push_str("  name = \"Untitled Project\",\n");
                script.push_str("  bpm = 120.0,\n");
                script.push_str("  sample_rate = 44100\n");
                script.push_str("}\n\n");

                for plugin in plugins {
                        // 插件导出：在 Lua 中以 `plugin { id = ..., name = ..., ... }` 形式记录
                        script.push_str(&format!(
                "plugin {{\n  id = \"{}\",\n  name = \"{}\",\n  label = \"{}\",\n  routing_track = {},\n  format = \"Internal\"\n}}\n\n",
                plugin.id, plugin.name, plugin.label, plugin.routing_track_index
            ));
                }

                for track in tracks {
                        // 轨道导出：记录基本属性以及指向混音目标轨道 id
                        script.push_str(&format!(
        "track {{\n  id = {},\n  name = \"{}\",\n  color = \"{}\",\n  target_mixer = {}\n}}\n\n",
        track.id, track.name, track.color, track.target_mixer_track_id
      ));
                }

                for clip in clips {
                        let mut audio_path_str = String::new();
                        // 如果该 clip 引用了音频文件，则拷贝到 `assets/` 目录并在 Lua 中记录相对路径
                        if let crate::daw::model::ClipContent::Audio { path: audio_src } = &clip.content {
                                // 将引用的音频复制到 `assets` 目录（保持文件名）
                                let assets_dir = project_path.join("assets");
                                if !assets_dir.exists() {
                                        fs::create_dir_all(&assets_dir).ok();
                                }

                                let src_path = Path::new(audio_src);
                                if let Some(file_name) = src_path.file_name() {
                                        let dest_path = assets_dir.join(file_name);
                                        if let Err(e) = fs::copy(src_path, &dest_path) {
                                                println!("Failed to copy asset: {}", e);
                                        }
                                        // 写入相对音频路径到导出脚本
                                        audio_path_str = format!("assets/{}", file_name.to_string_lossy());
                                }
                        }
                        // 内容类型：MIDI 或 Audio，导出为字符串
                        let content_type = match clip.content {
                                crate::daw::model::ClipContent::Midi => "midi",
                                crate::daw::model::ClipContent::Audio { .. } => "audio",
                        };

                        let inst_ids_str = clip
                                .instrument_ids
                                .iter()
                                .map(|id| format!("\"{}\"", id))
                                .collect::<Vec<_>>()
                                .join(", ");

                        let inst_routes_str = clip
                                .instrument_routes
                                .iter()
                                .map(|(k, v)| format!("[\"{}\"] = {}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ");

                        script.push_str(&format!("clip {{\n  id = \"{}\",\n  track_id = {},\n  name = \"{}\",\n  color = \"{}\",\n  start = {},\n  start_bar = {},\n  start_beat = {},\n  start_sixteenth = {},\n  start_tick = {},\n  duration = {},\n  duration_bars = {},\n  duration_beats = {},\n  duration_sixteenths = {},\n  duration_ticks = {},\n  duration_total_ticks = {},\n  type = \"{}\",\n  audio_path = \"{}\",\n  instrument_ids = {{{}}},\n  instrument_routes = {{{}}}\n}}\n\n",
                clip.id, clip.track_id, clip.name, clip.color,
                clip.start.time, clip.start.bar, clip.start.beat, clip.start.sixteenth, clip.start.tick,
                clip.length.seconds, clip.length.bars, clip.length.beats, clip.length.sixteenths, clip.length.ticks, clip.length.total_ticks,
                content_type, audio_path_str, inst_ids_str, inst_routes_str));
                }

                // 导出混音轨道（音量/声像/静音/独唱等状态）
                for mixer in mixer_tracks {
                        script.push_str(&format!("mixer_strip {{\n  id = {},\n  volume = {:.2},\n  pan = {:.2},\n  mute = {},\n  solo = {}\n}}\n\n",
                mixer.id, mixer.volume, mixer.pan, mixer.mute, mixer.solo));
                }

                script
        }

        /// 递归拷贝目录（保持目录结构）
        ///
        /// 该函数复制源目录下的所有文件和子目录到目标目录。遇到错误会返回 `anyhow::Error`，上层调用通常记录错误信息但不会中止整个项目保存流程。
        fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), anyhow::Error> {
                if !dst.exists() {
                        fs::create_dir_all(&dst)?;
                }
                for entry in fs::read_dir(src)? {
                        let entry = entry?;
                        let file_type = entry.file_type()?;
                        let from = entry.path();
                        let to = dst.join(entry.file_name());
                        if file_type.is_dir() {
                                Self::copy_dir_all(&from, &to)?;
                        } else if file_type.is_file() {
                                fs::copy(&from, &to)?;
                        }
                }
                Ok(())
        }

        /// 向上查找包含 `manifest.lua` 的插件根目录
        ///
        /// 从 `start` 路径开始向上遍历父目录，直到找到包含 `manifest.lua` 的目录并返回该目录路径。
        fn find_plugin_folder_with_manifest(start: &Path) -> Option<PathBuf> {
                let mut folder = start.to_path_buf();
                while folder.parent().is_some() {
                        if folder.join("manifest.lua").exists() {
                                return Some(folder);
                        }
                        if !folder.pop() {
                                break;
                        }
                }
                None
        }

        /// 递归复制父 manifest 的 `children` 字段声明的本地子插件
        ///
        /// 该函数会尝试解析 manifest.lua 中 `children` 表并复制每个 `backend.type = "local"` 的文件路径。
        /// - 如果子项是一个相对路径，则解析为相对于父 manifest 的路径
        /// - 如果子插件本身包含 manifest，则会将整个子插件文件夹复制到 `plugins/official`，并递归检查子 plugin 的 children
        /// - 如果找不到 manifest，会尝试把指定路径（文件名）复制到 `plugins/local`
        fn copy_children_from_manifest(manifest_folder: &Path, plugins_dir: &Path) -> Result<(), anyhow::Error> {
                let manifest_path = manifest_folder.join("manifest.lua");
                if !manifest_path.exists() {
                        return Ok(());
                }

                let content = match fs::read_to_string(&manifest_path) {
                        Ok(c) => c,
                        Err(_) => return Ok(()),
                };

                if let Ok(tbl) = Lua::new().load(&content).eval::<Table>() {
                        if let Ok(children_tbl) = tbl.get::<Table>("children") {
                                for pair in children_tbl.sequence_values::<Table>() {
                                        if let Ok(child_tbl) = pair {
                                                if let Ok(backend_tbl) = child_tbl.get::<Table>("backend") {
                                                        if let Ok(t) = backend_tbl.get::<String>("type") {
                                                                if t == "local" {
                                                                        if let Ok(p) = backend_tbl.get::<String>("path")
                                                                        {
                                                                                let child_path =
                                                                                        manifest_folder.join(&p);
                                                                                if let Some(child_folder) = Self::find_plugin_folder_with_manifest(&child_path)
                    {
                      // 检查子插件 manifest 的 `copy_on_project_save` 标志
                      let child_manifest = child_folder.join("manifest.lua");
                      let mut child_should_copy = true;
                      if child_manifest.exists() {
                        if let Ok(child_content) = fs::read_to_string(&child_manifest) {
                          if let Ok(child_tbl2) = Lua::new().load(&child_content).eval::<Table>() {
                            if let Ok(flag) = child_tbl2.get::<bool>("copy_on_project_save") {
                              child_should_copy = flag;
                            }
                          }
                        }
                      }

                      if child_should_copy {
                        let folder_name = child_folder
                          .file_name()
                          .map(|s| s.to_string_lossy().to_string())
                          .unwrap_or_else(|| "child_plugin".to_string());
                        let dest = plugins_dir.join("official").join(folder_name);
                        if let Err(e) = Self::copy_dir_all(&child_folder, &dest) {
                          println!("Failed to copy child plugin folder: {}", e);
                        }
                        // Recurse into child's manifest
                        let _ = Self::copy_children_from_manifest(&child_folder, plugins_dir);
                      } else {
                        // 子插件要求不复制；跳过
                      }
                    } else {
                      // 找不到 manifest：尝试复制指定的文件路径
                      if child_path.exists() {
                        let dest_dir = plugins_dir.join("local");
                        let _ = fs::create_dir_all(&dest_dir);
                        if let Some(fname) = child_path.file_name() {
                          let _ = fs::copy(&child_path, dest_dir.join(fname));
                        }
                      }
                    }
                                                                        }
                                                                }
                                                        }
                                                }
                                        }
                                }
                        }
                }

                Ok(())
        }

        /// 从磁盘路径载入项目并解析为 `ProjectSchema`。
        ///
        /// 流程概要：
        /// 1. 读取 `project.lua`，并在 Lua 环境中定义采集函数（project/track/clip/plugin/mixer_strip）来收集表数据
        /// 2. 执行 `project.lua`，从 _G.project_data 中取出 tables
        /// 3. 将 tables 转换为 `ProjectSchema`：tracks、clips、plugins 等
        /// 4. 从 `data.db` 中加载 `notes` 表并将它们归属到对应的 clip
        ///
        /// 备注：函数通过使用 `mlua` 读取 Lua 表以解析 Lua 脚本并通过 rusqlite 读取二进制/notes
        pub fn load_project(path: &Path) -> Result<ProjectSchema> {
                let lua = Lua::new();
                let script_path = path.join("project.lua");
                let script_content = fs::read_to_string(script_path)?;

                let globals = lua.globals();

                // 在 Lua 中定义采集函数，用于解析导出的 project.lua
                lua.load(r#"
            _G.project_data = {
                meta = {},
                tracks = {},
                clips = {},
                mixer = {},
                plugins = {}
            }

            function project(t)
                _G.project_data.meta = t
            end

            function track(t)
                table.insert(_G.project_data.tracks, t)
            end

            function clip(t)
                table.insert(_G.project_data.clips, t)
            end

            function mixer_strip(t)
                table.insert(_G.project_data.mixer, t)
            end

            function plugin(t)
                table.insert(_G.project_data.plugins, t)
            end
        "#)
                        .exec()
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                // 执行 project.lua，并收集表数据到 `_G.project_data`
                lua.load(&script_content)
                        .exec()
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                // 从 Lua 表中提取数据构造 `ProjectSchema`；注意：对字段访问要处理可能的缺失/默认值
                let project_data: Table = globals
                        .get("project_data")
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let meta: Table = project_data.get("meta").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let tracks_tbl: Vec<Table> = project_data.get("tracks").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let clips_tbl: Vec<Table> = project_data.get("clips").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let _mixer_tbl: Vec<Table> = project_data.get("mixer").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                let plugins_tbl: Vec<Table> = project_data
                        .get("plugins")
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                let mut schema = ProjectSchema {
                        meta: ProjectMetadata {
                                name: meta
                                        .get("name")
                                        .map_err(|e| anyhow::anyhow!(e.to_string()))
                                        .unwrap_or("Untitled".to_string()),
                                author: "Unknown".to_string(),
                                version: "0.0.1".to_string(),
                                created_at: 0,
                                updated_at: 0,
                                description: "".to_string(),
                        },
                        settings: ProjectSettings {
                                bpm: meta
                                        .get("bpm")
                                        .map_err(|e| anyhow::anyhow!(e.to_string()))
                                        .unwrap_or(120.0),
                                sample_rate: meta
                                        .get("sample_rate")
                                        .map_err(|e| anyhow::anyhow!(e.to_string()))
                                        .unwrap_or(44100),
                                time_signature: (4, 4),
                        },
                        tracks: vec![],
                        mixer: MixerSchema { tracks: vec![] },
                        plugins: vec![],
                };

                for t in tracks_tbl {
                        schema.tracks.push(TrackSchema {
                                id: t.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                name: t.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                color: t.get("color").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                track_type: TrackType::Audio, // Default
                                clips: vec![],                // Will populate later
                                target_mixer_track_id: t
                                        .get("target_mixer")
                                        .map_err(|e| anyhow::anyhow!(e.to_string()))?,
                        });
                }

                for plugin in plugins_tbl {
                        schema.plugins.push(PluginSchema {
                                id: plugin.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                name: plugin.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                label: plugin
                                        .get("label")
                                        .unwrap_or_else(|_| plugin.get("name").unwrap_or("Unknown".to_string())),
                                routing_track_index: plugin.get("routing_track").unwrap_or(0),
                                format: plugin.get("format").unwrap_or("Internal".to_string()),
                                state_blob_id: None, // 状态 blob 后续从 data.db 加载或按 id 匹配
                        });
                }

                // 从 `data.db` 加载 MIDI notes（按 clip_id 查询 note 数据）
                let db_path = path.join("data.db");
                let conn = Connection::open(&db_path)?;
                let mut stmt = conn.prepare("SELECT note, start, duration, velocity FROM notes WHERE clip_id = ?1")?;

                // 将 clips 映射到对应的 tracks
                for c in clips_tbl {
                        let track_id: usize = c.get("track_id").map_err(|e| anyhow::anyhow!(e.to_string()))?;
                        let clip_id: String = c.get("id").map_err(|e| anyhow::anyhow!(e.to_string()))?;

                        // 查询并收集该 clip 的 notes
                        let note_iter = stmt.query_map(params![clip_id], |row| {
                                Ok(NoteSchema {
                                        note: row.get(0)?,
                                        start: row.get(1)?,
                                        duration: row.get(2)?,
                                        velocity: row.get(3)?,
                                        channel: 0,
                                })
                        })?;

                        let mut notes = Vec::new();
                        for note in note_iter {
                                notes.push(note?);
                        }

                        let type_str: String = c.get("type").unwrap_or("midi".to_string());
                        let audio_path: String = c.get("audio_path").unwrap_or("".to_string());

                        let content_type = if type_str == "audio" {
                                // 将相对路径解析为绝对音频路径
                                let abs_path = path.join(&audio_path);
                                ClipContentType::Audio {
                                        file_path: abs_path.to_string_lossy().to_string(),
                                }
                        } else {
                                ClipContentType::Midi
                        };

                        let inst_ids_tbl: Option<Table> = c.get("instrument_ids").ok();
                        let mut instrument_ids = Vec::new();
                        if let Some(tbl) = inst_ids_tbl {
                                for pair in tbl.pairs::<usize, String>() {
                                        if let Ok((_, id)) = pair {
                                                instrument_ids.push(id);
                                        }
                                }
                        }

                        let inst_routes_tbl: Option<Table> = c.get("instrument_routes").ok();
                        let mut instrument_routes = std::collections::HashMap::new();
                        if let Some(tbl) = inst_routes_tbl {
                                for pair in tbl.pairs::<String, usize>() {
                                        if let Ok((k, v)) = pair {
                                                instrument_routes.insert(k, v);
                                        }
                                }
                        }

                        if let Some(track) = schema.tracks.iter_mut().find(|t| t.id == track_id) {
                                track.clips.push(ClipSchema {
                                        id: clip_id,
                                        name: c.get("name").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                        color: c.get("color").unwrap_or("#3b82f6".to_string()),
                                        start_position: c.get("start").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                        start_bar: c.get("start_bar").unwrap_or(0),
                                        start_beat: c.get("start_beat").unwrap_or(0),
                                        start_sixteenth: c.get("start_sixteenth").unwrap_or(0),
                                        start_tick: c.get("start_tick").unwrap_or(0),
                                        duration: c.get("duration").map_err(|e| anyhow::anyhow!(e.to_string()))?,
                                        duration_bars: c.get("duration_bars").unwrap_or(0),
                                        duration_beats: c.get("duration_beats").unwrap_or(0),
                                        duration_sixteenths: c.get("duration_sixteenths").unwrap_or(0),
                                        duration_ticks: c.get("duration_ticks").unwrap_or(0),
                                        duration_total_ticks: c.get("duration_total_ticks").unwrap_or(0),
                                        offset: 0.0,
                                        content_type,
                                        note_count: notes.len(),
                                        notes,
                                        instrument_ids,
                                        instrument_routes,
                                });
                        }
                }

                Ok(schema)
        }

        /// 从项目的 `data.db` 中读取已保存的插件状态（id -> blob）
        /// 从项目的 `data.db` 中读取已保存的插件状态（id -> blob）。
        ///
        /// 返回一个 HashMap，键为插件 id，值为状态二进制 blob。此接口常用于在加载项目后，把 blob 写回插件实例以恢复其内部状态。
        pub fn load_plugin_states(path: &Path) -> Result<std::collections::HashMap<String, Vec<u8>>> {
                let db_path = path.join("data.db");
                let conn = Connection::open(&db_path)?;
                let mut stmt = conn.prepare("SELECT id, state FROM plugins")?;

                let rows = stmt.query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
                })?;

                let mut states = std::collections::HashMap::new();
                for row in rows {
                        let (id, state) = row?;
                        states.insert(id, state);
                }
                Ok(states)
        }
}
