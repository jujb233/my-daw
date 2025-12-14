use anyhow::Result;
use mlua::Lua;
use mlua::Table;
use std::fs;
use std::path::{Path, PathBuf};

use crate::audio::plugins::manager::PluginSource;
use crate::daw::state::AppState;

pub fn copy_plugins_into_project(state: &AppState, project_path: &Path) {
        let plugin_mgr = state.plugin_manager.lock().unwrap();
        let plugins_dir = project_path.join("plugins");
        if !plugins_dir.exists() {
                fs::create_dir_all(&plugins_dir).ok();
        }

        let plugins = state.active_plugins.lock().unwrap();
        for plugin in plugins.iter() {
                match plugin_mgr.get_plugin_source(&plugin.id) {
                        PluginSource::Local(lib_path) => {
                                if let Some(folder) = find_plugin_folder_with_manifest(&lib_path) {
                                        let manifest_path = folder.join("manifest.lua");
                                        let mut should_copy = true;
                                        if manifest_path.exists() {
                                                if let Ok(content) = fs::read_to_string(&manifest_path) {
                                                        if let Ok(tbl) = Lua::new().load(&content).eval::<Table>() {
                                                                if let Ok(flag) =
                                                                        tbl.get::<bool>("copy_on_project_save")
                                                                {
                                                                        should_copy = flag;
                                                                }
                                                        }
                                                }
                                        }

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
                                                                if let Ok(p) = backend_tbl.get::<String>("path") {
                                                                        let child_path = manifest_folder.join(&p);
                                                                        if let Some(child_folder) =
                                                                                find_plugin_folder_with_manifest(
                                                                                        &child_path,
                                                                                )
                                                                        {
                                                                                // check copy flag
                                                                                let child_manifest = child_folder
                                                                                        .join("manifest.lua");
                                                                                let mut child_should_copy = true;
                                                                                if child_manifest.exists() {
                                                                                        if let Ok(child_content) =
                                                                                                fs::read_to_string(
                                                                                                        &child_manifest,
                                                                                                )
                                                                                        {
                                                                                                if let Ok(child_tbl2) = Lua::new().load(&child_content).eval::<Table>() {
                                                    if let Ok(flag) = child_tbl2.get::<bool>("copy_on_project_save") {
                                                        child_should_copy = flag;
                                                    }
                                                }
                                                                                        }
                                                                                }
                                                                                if child_should_copy {
                                                                                        let folder_name = child_folder.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "child_plugin".to_string());
                                                                                        let dest = plugins_dir
                                                                                                .join("official")
                                                                                                .join(folder_name);
                                                                                        if let Err(e) = copy_dir_all(
                                                                                                &child_folder,
                                                                                                &dest,
                                                                                        ) {
                                                                                                println!(
                                                                                                        "Failed to copy child plugin folder: {}",
                                                                                                        e
                                                                                                );
                                                                                        }
                                                                                        let _ = copy_children_from_manifest(&child_folder, plugins_dir);
                                                                                }
                                                                        } else {
                                                                                if child_path.exists() {
                                                                                        let dest_dir = plugins_dir
                                                                                                .join("local");
                                                                                        let _ = fs::create_dir_all(
                                                                                                &dest_dir,
                                                                                        );
                                                                                        if let Some(fname) =
                                                                                                child_path.file_name()
                                                                                        {
                                                                                                let _ = fs::copy(
                                                                                                        &child_path,
                                                                                                        dest_dir.join(
                                                                                                                fname,
                                                                                                        ),
                                                                                                );
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
