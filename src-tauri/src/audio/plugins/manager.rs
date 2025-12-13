use crate::audio::core::plugin::{Plugin, PluginInfo};
use crate::audio::plugins::clap::plugin::ClapPlugin;
use mlua::Lua;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PluginManager {
        known_plugins: HashMap<String, PluginInfo>,
        clap_paths: HashMap<String, String>,
        local_paths: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum PluginSource {
        Local(PathBuf),
        Clap(PathBuf),
        Builtin(Option<String>), // optional module name
        Unknown,
}

impl PluginManager {
        pub fn new() -> Self {
                let mut manager = Self {
                        known_plugins: HashMap::new(),
                        clap_paths: HashMap::new(),
                        local_paths: HashMap::new(),
                };
                manager.scan_native_plugins();
                manager
        }

        pub fn get_plugin_source(&self, unique_id: &str) -> PluginSource {
                if let Some(p) = self.local_paths.get(unique_id) {
                        return PluginSource::Local(PathBuf::from(p));
                }
                if let Some(p) = self.clap_paths.get(unique_id) {
                        return PluginSource::Clap(PathBuf::from(p));
                }
                // If known plugin and not in local/clap, consider builtin and try map to module name
                if let Some(info) = self.known_plugins.get(unique_id) {
                        if let crate::audio::core::plugin::PluginType::Native = info.plugin_type {
                                let module = Self::builtin_module_for_unique_id(unique_id);
                                return PluginSource::Builtin(module);
                        }
                }
                PluginSource::Unknown
        }

        fn builtin_module_for_unique_id(unique_id: &str) -> Option<String> {
                match unique_id {
                        "com.mydaw.simplesynth" => Some("simple_synth".to_string()),
                        "com.mydaw.wavegenerator" => Some("wave_generator".to_string()),
                        "com.mydaw.gainfader" => Some("gain_fader".to_string()),
                        "com.mydaw.levelmeter" => Some("level_meter".to_string()),
                        _ => None,
                }
        }

        fn scan_native_plugins(&mut self) {
                // Try several likely locations for the `plugins/official` folder.
                // When running from `src-tauri` the workspace-level `plugins` folder is one level up.
                let candidates = ["plugins/official", "../plugins/official"];
                for c in &candidates {
                        let p = Path::new(c);
                        if p.exists() && p.is_dir() {
                                self.scan_plugins_dir(p);
                                return;
                        }
                }
                // fallback: attempt to scan relative path anyway
                self.scan_plugins_dir(Path::new("plugins/official"));
        }

        fn create_builtin_by_module(_module: &str) -> Option<Box<dyn Plugin>> {
                // host does not instantiate builtin modules internally
                None
        }

        /// Scan a plugins directory (which contains plugin folders with `manifest.lua`) and
        /// register discovered plugins into the manager. Used for scanning both the host
        /// `plugins/official` and per-project `project/plugins` copies.
        pub fn scan_plugins_dir(&mut self, plugin_dir: &Path) {
                if plugin_dir.exists() && plugin_dir.is_dir() {
                        // try to canonicalize base dir for robust path resolution
                        let base = fs::canonicalize(plugin_dir).unwrap_or_else(|_| plugin_dir.to_path_buf());
                        if let Ok(entries) = fs::read_dir(&base) {
                                for entry in entries.flatten() {
                                        let mut plugin_folder = entry.path();
                                        // canonicalize plugin_folder if possible
                                        plugin_folder = fs::canonicalize(&plugin_folder).unwrap_or(plugin_folder);
                                        if plugin_folder.is_dir() {
                                                let manifest = plugin_folder.join("manifest.lua");
                                                if manifest.exists() {
                                                        if let Ok(content) = fs::read_to_string(&manifest) {
                                                                let lua = Lua::new();
                                                                // Expect manifest.lua to `return { ... }`
                                                                match lua.load(&content).eval::<mlua::Table>() {
                                                                        Ok(tbl) => {
                                                                                let id: Option<String> =
                                                                                        tbl.get("id").ok();
                                                                                let name: Option<String> =
                                                                                        tbl.get("name").ok();
                                                                                let backend_tbl: Option<mlua::Table> =
                                                                                        tbl.get("backend").ok();
                                                                                if let (
                                                                                        Some(id),
                                                                                        Some(name),
                                                                                        Some(backend),
                                                                                ) = (id, name, backend_tbl)
                                                                                {
                                                                                        let btype: Option<String> =
                                                                                                backend.get("type")
                                                                                                        .ok();
                                                                                        if let Some(t) = btype {
                                                                                                if t == "local" {
                                                                                                        let lib_path_str: Option<String> = backend.get("path").ok();
                                                                                                        if let Some(bp) = lib_path_str {
                                                                                                                // Resolve path relative to plugin folder and canonicalize
                                                                                                                let full = fs::canonicalize(plugin_folder.join(&bp)).unwrap_or(plugin_folder.join(&bp));
                                                                                                                self.local_paths.insert(id.clone(), full.to_string_lossy().to_string());
                                                                                                                // Try load plugin to get info
                                                                                                                match unsafe { crate::audio::core::ffi_plugin::FFIPlugin::new(self.local_paths.get(&id).unwrap(), 44100.0) } {
                                                                                                                        Ok(plugin) => {
                                                                                                                                let info = plugin.info();
                                                                                                                                let mut info = info.clone();
                                                                                                                                info.unique_id = id.clone();
                                                                                                                                info.name = name.clone();
                                                                                                                                self.known_plugins.insert(id.clone(), info);
                                                                                                                        }
                                                                                                                        Err(e) => {
                                                                                                                                println!("PluginManager: failed to load native plugin {} at {}: {}", id, full.to_string_lossy(), e);
                                                                                                                        }
                                                                                                                }
                                                                                                        }
                                                                                                } else if t == "builtin"
                                                                                                {
                                                                                                        let module: Option<String> = backend.get("module").ok();
                                                                                                        if let Some(m) =
                                                                                                                module
                                                                                                        {
                                                                                                                if let Some(plugin) = Self::create_builtin_by_module(&m) {
                                                                                                                        let info = plugin.info();
                                                                                                                        let mut info = info.clone();
                                                                                                                        info.unique_id = id.clone();
                                                                                                                        info.name = name.clone();
                                                                                                                        self.known_plugins.insert(id.clone(), info);
                                                                                                                }
                                                                                                        }
                                                                                                }
                                                                                        }
                                                                                }
                                                                        }
                                                                        Err(e) => {
                                                                                println!(
                                                                                        "PluginManager: failed to parse manifest {}: {}",
                                                                                        manifest.display(),
                                                                                        e
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

        /// Clear known state and re-scan official plugins directory.
        pub fn rescan(&mut self) {
                self.known_plugins.clear();
                self.clap_paths.clear();
                self.local_paths.clear();
                self.scan_native_plugins();
        }

        pub fn scan_clap_plugin(&mut self, path: &str) -> Result<PluginInfo, String> {
                unsafe {
                        let plugin = ClapPlugin::new(path)?;
                        let info = plugin.info();
                        self.known_plugins.insert(info.unique_id.clone(), info.clone());
                        self.clap_paths.insert(info.unique_id.clone(), path.to_string());
                        Ok(info)
                }
        }

        pub fn get_available_plugins(&self) -> Vec<PluginInfo> {
                self.known_plugins.values().cloned().collect()
        }

        pub fn create_plugin(&self, unique_id: &str) -> Option<Box<dyn Plugin>> {
                // Do not instantiate builtin implementations in host; attempt CLAP or local FFI libs.
                if let Some(path) = self.clap_paths.get(unique_id) {
                        unsafe {
                                if let Ok(plugin) = ClapPlugin::new(path) {
                                        return Some(Box::new(plugin));
                                }
                        }
                }

                if let Some(lib_path) = self.local_paths.get(unique_id) {
                        unsafe {
                                if let Ok(plugin) = crate::audio::core::ffi_plugin::FFIPlugin::new(lib_path, 44100.0) {
                                        return Some(Box::new(plugin));
                                }
                        }
                }

                None
        }

        pub fn get_plugin_parameters(
                &self,
                unique_id: &str,
        ) -> Option<Vec<crate::audio::core::plugin::PluginParameter>> {
                if let Some(plugin) = self.create_plugin(unique_id) {
                        return Some(plugin.get_parameters());
                }
                None
        }
}
