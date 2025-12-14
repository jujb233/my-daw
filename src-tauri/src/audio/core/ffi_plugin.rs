use crate::audio::core::plugin::{AudioBuffer, Plugin, PluginEvent, PluginInfo};
use libc;
use libloading::Library;
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::ptr::NonNull;

// 类型别名：便于阅读与维护
type CreateFn = unsafe extern "C" fn(f32) -> *mut c_void;
type DestroyFn = unsafe extern "C" fn(*mut c_void);
type ProcessFn = unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize);
type SetParamFn = unsafe extern "C" fn(*mut c_void, u32, f32);
type GetParamFn = unsafe extern "C" fn(*mut c_void, u32) -> f32;
type InfoFn = unsafe extern "C" fn() -> *mut c_char;
type FreeStrFn = unsafe extern "C" fn(*mut c_char);
type StateGetFn = unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut u8;
type StateFreeFn = unsafe extern "C" fn(*mut u8, usize);
type StateSetFn = unsafe extern "C" fn(*mut c_void, *const u8, usize);

#[allow(dead_code)]
pub struct FFIPlugin {
        // 持有动态库实例以保证函数指针调用时库仍然有效
        lib: Library,
        // C 插件实例指针（由插件的 create 函数返回），使用 NonNull 表示可选实例
        inst: Option<NonNull<c_void>>,
        // 插件销毁函数（必须）：在 Drop 时调用以释放实例
        destroy_fn: DestroyFn,
        // 插件处理回调（必须）：对传入样本缓冲区就地处理
        process_fn: ProcessFn,
        // 可选参数设置函数
        set_param_fn: Option<SetParamFn>,
        // 可选参数读取函数
        get_param_fn: Option<GetParamFn>,
        // 返回插件信息 JSON 的函数（返回 C 分配的字符串）
        info_fn: InfoFn,
        // 释放 info_fn 返回字符串的函数（由插件实现）
        free_str_fn: FreeStrFn,
        // 可选：返回序列化状态的函数（指针 + 长度）
        state_get_fn: Option<StateGetFn>,
        // 可选：释放 state_get_fn 返回内存的函数
        state_free_fn: Option<StateFreeFn>,
        // 可选：将序列化状态写回插件实例的函数
        state_set_fn: Option<StateSetFn>,
}

// 注意：FFIPlugin 持有指向 C 插件实例的裸指针与动态库句柄。
// 目前将其标记为 `Send`/`Sync`，前提是假定宿主以线程安全方式使用插件且插件不依赖线程本地状态。
// TODO: 未来应提供更安全的封装以确保不变式与内存安全。
unsafe impl Send for FFIPlugin {}
unsafe impl Sync for FFIPlugin {}

impl FFIPlugin {
        pub unsafe fn new(lib_path: &str, sample_rate: f32) -> Result<Self, anyhow::Error> {
                // 仅在这里进行符号解析（unsafe），把函数指针复制出来，之后的调用尽量把 unsafe 限制在小块内。
                let lib = unsafe { Library::new(lib_path)? };

                let create_sym = unsafe { lib.get::<CreateFn>(b"create_plugin")? };
                let destroy_sym = unsafe { lib.get::<DestroyFn>(b"destroy_plugin")? };
                let process_sym = unsafe { lib.get::<ProcessFn>(b"plugin_process")? };
                let info_sym = unsafe { lib.get::<InfoFn>(b"plugin_info_json")? };
                let free_str_sym = unsafe { lib.get::<FreeStrFn>(b"plugin_free_string")? };

                // optional symbols
                let set_param_sym = match unsafe { lib.get::<SetParamFn>(b"plugin_set_param") } {
                        Ok(s) => Some(*s),
                        Err(_) => None,
                };
                let get_param_sym = match unsafe { lib.get::<GetParamFn>(b"plugin_get_param") } {
                        Ok(s) => Some(*s),
                        Err(_) => None,
                };

                let state_get_sym = match unsafe { lib.get::<StateGetFn>(b"plugin_get_state") } {
                        Ok(s) => Some(*s),
                        Err(_) => None,
                };
                let state_free_sym = match unsafe { lib.get::<StateFreeFn>(b"plugin_free_state_blob") } {
                        Ok(s) => Some(*s),
                        Err(_) => None,
                };
                let state_set_sym = match unsafe { lib.get::<StateSetFn>(b"plugin_set_state") } {
                        Ok(s) => Some(*s),
                        Err(_) => None,
                };

                // 把函数指针复制出来，symbol 可被丢弃，而 Library 被保存在结构体中以保证库仍然加载
                let create_fn: CreateFn = *create_sym;
                let destroy_fn: DestroyFn = *destroy_sym;
                let process_fn: ProcessFn = *process_sym;
                let info_fn: InfoFn = *info_sym;
                let free_str_fn: FreeStrFn = *free_str_sym;

                let set_param_fn = set_param_sym;
                let get_param_fn = get_param_sym;
                let state_get_fn = state_get_sym;
                let state_free_fn = state_free_sym;
                let state_set_fn = state_set_sym;

                // 调用插件创建实例（unsafe 调用外部函数）并用 NonNull 封装
                let raw_inst = unsafe { create_fn(sample_rate) };
                let inst = NonNull::new(raw_inst).map(|p| p.cast());

                Ok(Self {
                        lib,
                        inst,
                        destroy_fn,
                        process_fn,
                        set_param_fn,
                        get_param_fn,
                        info_fn,
                        free_str_fn,
                        state_get_fn,
                        state_free_fn,
                        state_set_fn,
                })
        }
        // 小型安全包装器：返回 info 字符串（若插件返回 null 则返回 None）
        fn info_string(&self) -> Option<String> {
                unsafe {
                        let raw = (self.info_fn)();
                        if raw.is_null() {
                                return None;
                        }
                        let s = CStr::from_ptr(raw).to_string_lossy().into_owned();
                        (self.free_str_fn)(raw);
                        Some(s)
                }
        }

        fn get_info(&self) -> PluginInfo {
                if let Some(json) = self.info_string() {
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

impl Drop for FFIPlugin {
        // 在 Rust 端释放时调用插件提供的 destroy 函数以清理 C 端资源
        fn drop(&mut self) {
                if let Some(inst) = self.inst {
                        unsafe { (self.destroy_fn)(inst.as_ptr()) }
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
                // 在这里把 unsafe 限制在小块内
                if let Some(get_fn) = &self.state_get_fn {
                        let mut len: usize = 0;
                        let ptr = unsafe {
                                (get_fn)(
                                        self.inst.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()),
                                        &mut len as *mut usize,
                                )
                        };
                        if ptr.is_null() || len == 0 {
                                return Vec::new();
                        }
                        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
                        let v = slice.to_vec();
                        if let Some(free_fn) = &self.state_free_fn {
                                unsafe { (free_fn)(ptr, len) }
                        } else {
                                unsafe { libc::free(ptr as *mut libc::c_void) }
                        }
                        return v;
                }
                Vec::new()
        }

        fn set_state(&mut self, _state: &[u8]) {
                if let Some(set_fn) = &self.state_set_fn {
                        unsafe {
                                (set_fn)(
                                        self.inst.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()),
                                        _state.as_ptr(),
                                        _state.len(),
                                )
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
                if let Some(inst) = self.inst {
                        unsafe {
                                (self.process_fn)(
                                        inst.as_ptr(),
                                        buffer.samples.as_mut_ptr(),
                                        frames,
                                        buffer.channels,
                                )
                        }
                }
        }

        // 读取参数（如果插件导出该接口），否则返回默认值 0.0
        fn get_param(&self, id: u32) -> f32 {
                if let Some(get_fn) = &self.get_param_fn {
                        unsafe {
                                (get_fn)(
                                        self.inst.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()),
                                        id,
                                )
                        }
                } else {
                        0.0
                }
        }

        // 写入参数（若插件未实现此函数则忽略）
        fn set_param(&mut self, id: u32, value: f32) {
                if let Some(set_fn) = &self.set_param_fn {
                        unsafe {
                                (set_fn)(
                                        self.inst.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut()),
                                        id,
                                        value,
                                )
                        }
                }
        }
}
