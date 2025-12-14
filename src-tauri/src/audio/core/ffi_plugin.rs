use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent, PluginInfo};
use libc;
use libloading::{Library, Symbol};
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;

#[allow(dead_code)]
pub struct FFIPlugin {
        // 泄漏的库引用（'static），用于保存从库中解析的符号引用
        #[allow(dead_code)]
        lib: &'static Library,
        // C 插件实例指针（由插件的 create 函数返回）
        inst: *mut c_void,
        // 插件销毁函数（必须）：在 Drop 时调用以释放实例
        destroy_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void)>,
        // 插件处理回调（必须）：对传入样本缓冲区就地处理
        process_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize)>,
        // 可选参数设置函数
        set_param_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, u32, f32)>>,
        // 可选参数读取函数
        get_param_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, u32) -> f32>>,
        // 返回插件信息 JSON 的函数（返回 C 分配的字符串）
        info_fn: Symbol<'static, unsafe extern "C" fn() -> *mut c_char>,
        // 释放 info_fn 返回字符串的函数（由插件实现）
        free_str_fn: Symbol<'static, unsafe extern "C" fn(*mut c_char)>,
        // 可选：返回序列化状态的函数（指针 + 长度）
        state_get_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut u8>>,
        // 可选：释放 state_get_fn 返回内存的函数
        state_free_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut u8, usize)>>,
        // 可选：将序列化状态写回插件实例的函数
        state_set_fn: Option<Symbol<'static, unsafe extern "C" fn(*mut c_void, *const u8, usize)>>,
}

// 注意：FFIPlugin 持有指向 C 插件实例的裸指针与动态库句柄。
// 目前将其标记为 `Send`/`Sync`，前提是假定宿主以线程安全方式使用插件且插件不依赖线程本地状态。
// TODO: 未来应提供更安全的封装以确保不变式与内存安全。
unsafe impl Send for FFIPlugin {}
unsafe impl Sync for FFIPlugin {}

impl FFIPlugin {
        pub unsafe fn new(lib_path: &str, sample_rate: f32) -> Result<Self, anyhow::Error> {
                let lib = unsafe { Library::new(lib_path)? };
                // 将 Library 泄漏以便保存符号引用（注意：泄漏会阻止库卸载）
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

                // 调用插件的 create 函数创建实例（工厂），返回 C 指针
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
                        // 从插件获取描述字符串（通常为 JSON），并由插件提供释放函数释放该字符串
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

                        // 解析最小字段（容错），用于 UI 显示
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
        // 在 Rust 端释放时调用插件提供的 destroy 函数以清理 C 端资源
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
                // 当前没有可序列化的参数信息，返回空列表
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
                                        // 回退：尝试使用 libc 释放指针
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

        // 将音频缓冲区交给插件处理（就地修改 samples）
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

        // 读取参数（如果插件导出该接口），否则返回默认值 0.0
        fn get_param(&self, id: u32) -> f32 {
                unsafe {
                        if let Some(get_fn) = &self.get_param_fn {
                                (get_fn)(self.inst, id)
                        } else {
                                0.0
                        }
                }
        }

        // 写入参数（若插件未实现此函数则忽略）
        fn set_param(&mut self, id: u32, value: f32) {
                unsafe {
                        if let Some(set_fn) = &self.set_param_fn {
                                (set_fn)(self.inst, id, value)
                        }
                }
        }
}
