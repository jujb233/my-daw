use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent, PluginInfo};
use libc;
use libloading::{Library, Symbol};
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;

#[allow(dead_code)]
pub struct FFIPlugin {
        #[allow(dead_code)]
        lib: &'static Library,
        inst: *mut c_void,
        destroy_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void)>,
        process_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize)>,
        set_param_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, u32, f32)>>,
        get_param_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, u32) -> f32>>,
        info_fn: Symbol<'static, unsafe extern "C" fn() -> *mut c_char>,
        free_str_fn: Symbol<'static, unsafe extern "C" fn(*mut c_char)>,
        state_get_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut u8>>,
        state_free_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut u8, usize)>>,
        state_set_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, *const u8, usize)>>,
}

// Safety: FFIPlugin contains raw pointers to C plugin instance and a dynamic library handle.
// We assert Send and Sync for now under the assumption that the plugin is only used by the host
// in a properly synchronized manner and does not use thread-local state in an undefined way.
// TODO: Revisit and provide a safe wrapper that ensures usable guarantees.
unsafe impl Send for FFIPlugin {}
unsafe impl Sync for FFIPlugin {}

impl FFIPlugin {
        pub unsafe fn new(lib_path: &str, sample_rate: f32) -> Result<Self, anyhow::Error> {
                let lib = unsafe { Library::new(lib_path)? };
                // Leak the Library to obtain a 'static reference so Symbols can be 'static
                let lib_box = Box::new(lib);
                let lib_ref: &'static Library = Box::leak(lib_box);

                let create: Symbol<unsafe extern "C" fn(f32) -> *mut c_void> =
                        unsafe { lib_ref.get(b"create_plugin")? };
                let destroy: Symbol<unsafe extern "C" fn(*mut c_void)> = unsafe { lib_ref.get(b"destroy_plugin")? };
                let process: Symbol<unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize)> =
                        unsafe { lib_ref.get(b"plugin_process")? };
                let set_param = match unsafe {
                        lib_ref.get::<unsafe extern "C" fn(*mut c_void, u32, f32)>(b"plugin_set_param")
                } {
                        Ok(s) => Some(s),
                        Err(_) => None,
                };
                let get_param = match unsafe {
                        lib_ref.get::<unsafe extern "C" fn(*mut c_void, u32) -> f32>(b"plugin_get_param")
                } {
                        Ok(s) => Some(s),
                        Err(_) => None,
                };
                let info: Symbol<unsafe extern "C" fn() -> *mut c_char> = unsafe { lib_ref.get(b"plugin_info_json")? };
                let free_str: Symbol<unsafe extern "C" fn(*mut c_char)> =
                        unsafe { lib_ref.get(b"plugin_free_string")? };

                // Optional state functions
                let state_get = match unsafe {
                        lib_ref.get::<unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut u8>(b"plugin_get_state")
                } {
                        Ok(s) => Some(s),
                        Err(_) => None,
                };
                let state_free =
                        match unsafe { lib_ref.get::<unsafe extern "C" fn(*mut u8, usize)>(b"plugin_free_state_blob") }
                        {
                                Ok(s) => Some(s),
                                Err(_) => None,
                        };
                let state_set = match unsafe {
                        lib_ref.get::<unsafe extern "C" fn(*mut c_void, *const u8, usize)>(b"plugin_set_state")
                } {
                        Ok(s) => Some(s),
                        Err(_) => None,
                };

                // Call create
                let inst = unsafe { create(sample_rate) };

                Ok(Self {
                        lib: lib_ref,
                        inst,
                        destroy_fn: destroy,
                        process_fn: process,
                        set_param_fn: set_param,
                        get_param_fn: get_param,
                        info_fn: info,
                        free_str_fn: free_str,
                        state_get_fn: state_get,
                        state_free_fn: state_free,
                        state_set_fn: state_set,
                })
        }

        fn get_info(&self) -> PluginInfo {
                unsafe {
                        let raw = (self.info_fn)();
                        if raw.is_null() {
                                return PluginInfo {
                                        name: "unknown".to_string(),
                                        vendor: "unknown".to_string(),
                                        url: "".to_string(),
                                        plugin_type: crate::audio::core::plugin::PluginType::Native,
                                        unique_id: "".to_string(),
                                };
                        }
                        let cstr = CStr::from_ptr(raw);
                        let json = cstr.to_string_lossy().to_string();
                        (self.free_str_fn)(raw);
                        // Parse JSON minimal info
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                                let id = v["id"].as_str().unwrap_or("").to_string();
                                let name = v["name"].as_str().unwrap_or("").to_string();
                                return PluginInfo {
                                        name,
                                        vendor: "My DAW".to_string(),
                                        url: "".to_string(),
                                        plugin_type: crate::audio::core::plugin::PluginType::Native,
                                        unique_id: id,
                                };
                        }
                        PluginInfo {
                                name: "unknown".to_string(),
                                vendor: "unknown".to_string(),
                                url: "".to_string(),
                                plugin_type: crate::audio::core::plugin::PluginType::Native,
                                unique_id: "".to_string(),
                        }
                }
        }
}

impl Drop for FFIPlugin {
        fn drop(&mut self) {
                unsafe {
                        (self.destroy_fn)(self.inst);
                }
        }
}

impl Plugin for FFIPlugin {
        fn info(&self) -> PluginInfo {
                self.get_info()
        }

        fn get_parameters(&self) -> Vec<crate::audio::core::plugin::PluginParameter> {
                // For now we have no serialized params; return empty
                vec![]
        }

        fn get_state(&self) -> Vec<u8> {
                unsafe {
                        if let Some(get_fn) = &self.state_get_fn {
                                let mut len: usize = 0;
                                let ptr = (get_fn)(self.inst, &mut len as *mut usize);
                                if ptr.is_null() || len == 0 {
                                        return Vec::new();
                                }
                                let slice = std::slice::from_raw_parts(ptr, len);
                                let v = slice.to_vec();
                                if let Some(free_fn) = &self.state_free_fn {
                                        (free_fn)(ptr, len);
                                } else {
                                        // fallback: attempt to free with libc
                                        libc::free(ptr as *mut libc::c_void);
                                }
                                return v;
                        }
                }
                Vec::new()
        }

        fn set_state(&mut self, _state: &[u8]) {
                unsafe {
                        if let Some(set_fn) = &self.state_set_fn {
                                (set_fn)(self.inst, _state.as_ptr(), _state.len());
                        }
                }
        }

        fn get_io_config(&self) -> crate::audio::core::plugin::IOConfig {
                crate::audio::core::plugin::IOConfig::default()
        }

        fn process(
                &mut self,
                buffer: &mut AudioBuffer,
                _events: &[PluginEvent],
                _output_events: &mut Vec<PluginEvent>,
        ) {
                let frames = buffer.samples.len() / buffer.channels;
                unsafe {
                        (self.process_fn)(
                                self.inst,
                                buffer.samples.as_mut_ptr(),
                                frames,
                                buffer.channels,
                        )
                }
        }

        fn get_param(&self, id: u32) -> f32 {
                unsafe {
                        if let Some(get_fn) = &self.get_param_fn {
                                (get_fn)(self.inst, id)
                        } else {
                                0.0
                        }
                }
        }

        fn set_param(&mut self, id: u32, value: f32) {
                unsafe {
                        if let Some(set_fn) = &self.set_param_fn {
                                (set_fn)(self.inst, id, value)
                        }
                }
        }
}
