use anyhow::Result;
use mlua::Lua;
use mlua::Table;
use std::fs;
use std::path::{Path, PathBuf};

use crate::audio::plugins::manager::PluginSource;
use crate::daw::state::AppState;

// 本文件负责在保存项目时将当前激活的插件复制到项目目录。
// 主要行为：遍历当前激活的插件，按来源（Local/Clap/Builtin）进行不同处理，
// 并遵循插件 `manifest.lua` 中 `copy_on_project_save` 的标志决定是否复制。
//
// 注：复制结果放于项目目录下的 `plugins` 子目录，进一步分为 `official`、`local`、`clap` 等子目录。

/// 将当前激活插件复制到指定项目路径下的 `plugins` 目录。
///
/// - 遵循插件 manifest 中的 `copy_on_project_save`（默认 true）决定是否复制完整文件夹。
/// - Local 类型优先查找包含 `manifest.lua` 的文件夹（视为官方插件），否则将二进制复制到 `local`。
pub fn copy_plugins_into_project(state: &AppState, project_path: &Path) {
        let plugin_mgr = state.plugin_manager.lock().unwrap();
        let plugins_dir = project_path.join("plugins");
        if !plugins_dir.exists() {
                fs::create_dir_all(&plugins_dir).ok();
        }

        let plugins = state.active_plugins.lock().unwrap();
        for plugin in plugins.iter() {
                match plugin_mgr.get_plugin_source(&plugin.id) {
                        // 本分支处理本地（Local）插件：可能是带 manifest 的文件夹（官方样式），
                        // 也可能是单个库文件（此时我们把其拷贝到 local 并生成一个简单 manifest）。
                        PluginSource::Local(lib_path) => {
                                if let Some(folder) = find_plugin_folder_with_manifest(&lib_path) {
                                        let manifest_path = folder.join("manifest.lua");
                                        let should_copy = manifest_copy_flag(&manifest_path);

                                        if should_copy {
                                                let folder_name = folder
                                                        .file_name()
                                                        .map(|s| s.to_string_lossy().to_string())
                                                        .unwrap_or(plugin.id.clone());
                                                let dest = plugins_dir.join("official").join(folder_name);
                                                if let Err(e) = copy_dir_all(&folder, &dest) {
                                                        println!("Failed to copy plugin folder: {}", e);
                                                }
                                                let _ = copy_children_from_manifest(&folder, &plugins_dir);
                                        }
                                } else {
                                        // no manifest: copy binary into local
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
                        // 处理 CLAP 插件：如果是文件夹则递归复制，否则直接复制到目标路径。
                        PluginSource::Clap(p) => {
                                let folder_name = p
                                        .file_name()
                                        .map(|s| s.to_string_lossy().to_string())
                                        .unwrap_or(plugin.id.clone());
                                let dest = plugins_dir.join("clap").join(folder_name);
                                if p.is_dir() {
                                        if let Err(e) = copy_dir_all(&p, &dest) {
                                                println!("Failed to copy clap plugin: {}", e);
                                        }
                                } else {
                                        if let Some(parent) = dest.parent() {
                                                let _ = fs::create_dir_all(parent);
                                        }
                                        let _ = fs::copy(&p, &dest);
                                }
                        }
                        // 内置插件：创建一个只包含 backend 信息的 manifest 写入 official/<id>/manifest.lua
                        PluginSource::Builtin(module_opt) => {
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
                        }
                        PluginSource::Unknown => {}
                }
        }
}

/// 递归复制目录内容到目标路径。
///
/// - `src`：源目录路径。
/// - `dst`：目标目录路径（若不存在则创建）。
///
/// 复制时会递归处理子目录与文件，遇到 IO 错误会向上返回 `Err`。
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
                        copy_dir_all(&from, &to)?;
                } else if file_type.is_file() {
                        fs::copy(&from, &to)?;
                }
        }
        Ok(())
}

/// 从起始路径向上查找包含 `manifest.lua` 的父目录。
///
/// - 如果 `start` 本身或其某个父目录包含 `manifest.lua`，返回该目录路径。
/// - 未找到则返回 `None`。
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
/// 读取 `manifest.lua` 并返回 `copy_on_project_save` 标志（默认 true）。
///
/// - 如果 manifest 不存在或读取/解析失败，函数默认返回 `true`，表示应复制。
fn manifest_copy_flag(manifest_path: &Path) -> bool {
        if !manifest_path.exists() {
                return true;
        }
        if let Ok(content) = fs::read_to_string(manifest_path) {
                if let Ok(tbl) = Lua::new().load(&content).eval::<Table>() {
                        if let Ok(flag) = tbl.get::<bool>("copy_on_project_save") {
                                return flag;
                        }
                }
        }
        true
}
/// 读取指定 `manifest_folder` 的 `manifest.lua`，处理其 `children` 表并将本地子插件复制到项目目录。
///
/// 行为要点：
/// - 仅处理 `backend.type == "local"` 的子项；
/// - 若子项路径指向包含 `manifest.lua` 的文件夹，则按 `copy_on_project_save` 决定是否拷贝整个子文件夹到 `official`；
/// - 否则若子项路径存在且为单个文件，则复制到 `plugins/local`。
fn copy_children_from_manifest(manifest_folder: &Path, plugins_dir: &Path) -> Result<(), anyhow::Error> {
        let manifest_path = manifest_folder.join("manifest.lua");
        if !manifest_path.exists() {
                return Ok(());
        }
        let content = match fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => return Ok(()),
        };
        let lua = Lua::new();
        let tbl = match lua.load(&content).eval::<Table>() {
                Ok(t) => t,
                Err(_) => return Ok(()),
        };
        let children_tbl = match tbl.get::<Table>("children") {
                Ok(c) => c,
                Err(_) => return Ok(()),
        };
        for pair in children_tbl.sequence_values::<Table>() {
                let child_tbl = match pair {
                        Ok(t) => t,
                        Err(_) => continue,
                };
                let backend_tbl = match child_tbl.get::<Table>("backend") {
                        Ok(b) => b,
                        Err(_) => continue,
                };
                let t = match backend_tbl.get::<String>("type") {
                        Ok(s) => s,
                        Err(_) => continue,
                };
                if t != "local" {
                        continue;
                }
                let p = match backend_tbl.get::<String>("path") {
                        Ok(s) => s,
                        Err(_) => continue,
                };
                let child_path = manifest_folder.join(&p);
                if let Some(child_folder) = find_plugin_folder_with_manifest(&child_path) {
                        let child_manifest = child_folder.join("manifest.lua");
                        if !manifest_copy_flag(&child_manifest) {
                                continue;
                        }
                        let folder_name = child_folder
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "child_plugin".to_string());
                        let dest = plugins_dir.join("official").join(folder_name);
                        if let Err(e) = copy_dir_all(&child_folder, &dest) {
                                println!("Failed to copy child plugin folder: {}", e);
                        }
                        let _ = copy_children_from_manifest(&child_folder, plugins_dir);
                } else if child_path.exists() {
                        let dest_dir = plugins_dir.join("local");
                        let _ = fs::create_dir_all(&dest_dir);
                        if let Some(fname) = child_path.file_name() {
                                let _ = fs::copy(&child_path, dest_dir.join(fname));
                        }
                }
        }
        Ok(())
}
