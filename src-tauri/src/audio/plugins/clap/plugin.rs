use crate::audio::core::plugin::{
    AudioBuffer, IOConfig, Plugin, PluginEvent, PluginInfo, PluginParameter, PluginType,
};
use clap_sys::entry::clap_plugin_entry;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::clap_process;
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Arc;
// use std::ptr::NonNull;
use std::os::raw::c_char;

// CLAP 主机（Host）的最小实现
unsafe extern "C" fn host_get_extension(
    _host: *const clap_host,
    _extension_id: *const c_char,
) -> *const std::ffi::c_void {
    ptr::null()
}

unsafe extern "C" fn host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn host_request_callback(_host: *const clap_host) {}

// 为了保证 clap_host 所需的 C 字符串指针在插件生命周期内有效，
// 我们把 clap_host 和对应的 CString 字段包装在一起并保存在 `ClapPlugin` 中。
struct ClapHost {
    host: clap_host,
    _name: CString,
    _vendor: CString,
    _url: CString,
    _version: CString,
}

pub struct ClapPlugin {
    _library: Arc<Library>, // 保持库加载（防止被卸载）
    plugin: Option<*mut clap_plugin>,
    info: PluginInfo,
    #[allow(dead_code)]
    io_config: IOConfig,
    params: Vec<PluginParameter>,
    // 保持 host 的生命周期与插件一致
    _host: Box<ClapHost>,
    // 预分配输出缓冲，避免实时线程分配
    out_l: Vec<f32>,
    out_r: Vec<f32>,
    max_frames: usize,
}
// 在此集中管理 Send/Sync 的不安全声明：
// 只有在确保底层 CLAP 插件在宿主中按需使用且宿主对线程访问做了约束时，这样做才是安全的。
// 这比在多个文件散落 unsafe impl 更容易审计。
unsafe impl Send for ClapPlugin {}
unsafe impl Sync for ClapPlugin {}

impl ClapPlugin {
    pub unsafe fn new(path: &str) -> Result<Self, String> {
        unsafe {
            let lib = Library::new(path).map_err(|e| e.to_string())?;
            let lib = Arc::new(lib);

            let entry_fn: Symbol<
                unsafe extern "C" fn(path: *const std::os::raw::c_char) -> *const clap_plugin_entry,
            > = lib.get(b"clap_entry").map_err(|e| e.to_string())?;

            let c_path = CString::new(path).map_err(|e| e.to_string())?;
            let entry = entry_fn(c_path.as_ptr());

            if entry.is_null() {
                return Err("Failed to get clap_entry".to_string());
            }

            // 安全封装：读取 entry 中的函数指针并检查
            let init = (*entry).init.ok_or("No init function")?;
            if !init(c_path.as_ptr()) {
                return Err("Failed to init clap plugin".to_string());
            }

            let get_factory = (*entry).get_factory.ok_or("No get_factory function")?;
            let factory_id = b"clap.plugin-factory\0";
            let factory = get_factory(factory_id.as_ptr() as *const _)
                as *const clap_sys::factory::plugin_factory::clap_plugin_factory;

            if factory.is_null() {
                return Err("Failed to get plugin factory".to_string());
            }

            let get_plugin_count = (*factory).get_plugin_count.ok_or("No get_plugin_count")?;
            let count = get_plugin_count(factory);
            if count == 0 {
                return Err("No plugins found in library".to_string());
            }

            // 目前只取第一个插件
            let get_plugin_descriptor = (*factory)
                .get_plugin_descriptor
                .ok_or("No get_plugin_descriptor")?;
            let descriptor = get_plugin_descriptor(factory, 0);
            if descriptor.is_null() {
                return Err("Failed to get plugin descriptor".to_string());
            }

            // 创建 host 并保证 CString 的生命周期
            let host_name =
                CString::new("MyDAW").unwrap_or_else(|_| CString::new("MyDAW").unwrap());
            let host_vendor =
                CString::new("MyDAW").unwrap_or_else(|_| CString::new("MyDAW").unwrap());
            let host_url = CString::new("").unwrap_or_else(|_| CString::new("").unwrap());
            let host_version =
                CString::new("0.1.0").unwrap_or_else(|_| CString::new("0.1.0").unwrap());

            let host_struct = clap_host {
                clap_version: clap_sys::version::CLAP_VERSION,
                host_data: ptr::null_mut(),
                name: host_name.as_ptr() as *const _,
                vendor: host_vendor.as_ptr() as *const _,
                url: host_url.as_ptr() as *const _,
                version: host_version.as_ptr() as *const _,
                get_extension: Some(host_get_extension),
                request_restart: Some(host_request_restart),
                request_process: Some(host_request_process),
                request_callback: Some(host_request_callback),
            };

            let boxed_host = Box::new(ClapHost {
                host: host_struct,
                _name: host_name,
                _vendor: host_vendor,
                _url: host_url,
                _version: host_version,
            });

            let plugin_id = (*descriptor).id;
            let create_plugin = (*factory).create_plugin.ok_or("No create_plugin")?;

            // create_plugin 需要一个指向 clap_host 的指针
            let plugin_ptr = create_plugin(factory, &boxed_host.host, plugin_id);
            if plugin_ptr.is_null() {
                return Err("Failed to create plugin instance".to_string());
            }

            // 把裸指针存为可选裸指针
            let plugin_ptr_mut = plugin_ptr as *mut clap_plugin;
            if plugin_ptr_mut.is_null() {
                return Err("create_plugin returned null pointer".to_string());
            }

            // 调用 init 与 activate（unsafe 调用集中）
            let init_plugin = (*plugin_ptr_mut).init.ok_or("No plugin init")?;
            if !init_plugin(plugin_ptr_mut) {
                return Err("Failed to initialize plugin instance".to_string());
            }

            let activate = (*plugin_ptr_mut).activate.ok_or("No activate")?;
            // 使用采样率和块大小进行激活
            // TODO: 获取真实的采样率
            if !activate(plugin_ptr_mut, 44100.0, 32, 4096) {
                return Err("Failed to activate plugin".to_string());
            }

            let name = {
                let p = (*descriptor).name;
                if p.is_null() {
                    "".to_string()
                } else {
                    CStr::from_ptr(p).to_string_lossy().into_owned()
                }
            };
            let vendor = {
                let p = (*descriptor).vendor;
                if p.is_null() {
                    "".to_string()
                } else {
                    CStr::from_ptr(p).to_string_lossy().into_owned()
                }
            };
            let unique_id = {
                if plugin_id.is_null() {
                    "".to_string()
                } else {
                    CStr::from_ptr(plugin_id).to_string_lossy().into_owned()
                }
            };

            // TODO: 扫描参数
            let params = Vec::new();

            // 预分配缓冲大小（以 frames 为单位），与 activate 中的 max_events/frames 保持一致
            let max_frames = 4096usize;
            let out_l = vec![0.0f32; max_frames];
            let out_r = vec![0.0f32; max_frames];

            Ok(Self {
                _library: lib,
                plugin: Some(plugin_ptr_mut),
                info: PluginInfo {
                    name,
                    vendor,
                    url: "".to_string(),
                    plugin_type: PluginType::Clap,
                    unique_id,
                },
                io_config: IOConfig {
                    inputs: 0,
                    outputs: 2,
                }, // 目前假设为立体声输出
                params,
                _host: boxed_host,
                out_l,
                out_r,
                max_frames,
            })
        }
    }
}

impl Drop for ClapPlugin {
    fn drop(&mut self) {
        // 将所有对 C 指针的交互集中到这里，并且进行空指针检查。
        if let Some(p) = self.plugin {
            unsafe {
                if let Some(deactivate) = (*p).deactivate {
                    deactivate(p);
                }
                if let Some(destroy) = (*p).destroy {
                    destroy(p);
                }
            }
        }
        // boxed host 和 library 将自动释放
    }
}

impl Plugin for ClapPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn get_parameters(&self) -> Vec<PluginParameter> {
        self.params.clone()
    }

    fn get_io_config(&self) -> IOConfig {
        self.io_config.clone()
    }

    fn process(
        &mut self,
        buffer: &mut AudioBuffer,
        _events: &[PluginEvent],
        _output_events: &mut Vec<PluginEvent>,
    ) {
        // 把 unsafe 使用限制在最小范围：先取出裸指针并在单个 unsafe 块内调用
        if let Some(p) = self.plugin {
            unsafe {
                let process_fn = (*p).process.unwrap();

                // 准备音频缓冲区
                // CLAP 期望接收指向 f32 数组的指针数组（用于非交错/非 interleaved）
                // 我们的 AudioBuffer 是交错还是非交错？
                // `AudioBuffer` 结构的 `samples: &mut [f32]`，在我们的简单引擎中通常是交错的。
                // 但 CLAP 通常要求非交错格式。
                // 如果我们的引擎是交错的，我们需要进行去交错处理（de-interleave）。
                // 暂时假设仅支持单声道或采用一些 hack 处理。
                // 注意，`plugin.rs` 中的 `AudioBuffer` 有 `channels` 字段。

                // 如果是交错格式，我们需要用于去交错的临时缓冲区。
                // 暂时先假设我们只是传递指针。
                // 但如果没有提前准备，在音频线程中实现去交错（且不分配）并不容易。

                // 对于原型，假设为立体声并在足够小时将其去交错到栈数组，
                // 否则如果无法处理则直接输出静音。

                // 实际上，这里先实现一个占位（dummy）处理以证明插件能加载运行。
                // 真正的音频处理需要更复杂的缓冲区管理。

                // 准备输出缓冲：使用预分配缓冲，避免实时分配
                let frames = buffer.samples.len() / 2;
                if frames > self.max_frames {
                    // 如果输入帧数超过预分配的限制，安全起见直接清零输出并返回
                    for s in buffer.samples.iter_mut() {
                        *s = 0.0;
                    }
                    return;
                }

                let out_l = &mut self.out_l[..frames];
                let out_r = &mut self.out_r[..frames];

                // 清零输出缓冲（插件负责写入）
                for v in out_l.iter_mut() {
                    *v = 0.0;
                }
                for v in out_r.iter_mut() {
                    *v = 0.0;
                }

                let mut out_ptrs = [out_l.as_mut_ptr(), out_r.as_mut_ptr()];

                let mut audio_out = clap_sys::audio_buffer::clap_audio_buffer {
                    data32: out_ptrs.as_mut_ptr(),
                    data64: ptr::null_mut(),
                    channel_count: 2,
                    latency: 0,
                    constant_mask: 0,
                };

                let process_data = clap_process {
                    steady_time: -1,
                    frames_count: frames as u32,
                    transport: ptr::null(),
                    audio_inputs: ptr::null_mut(), // 无输入
                    audio_outputs: &mut audio_out,
                    audio_inputs_count: 0,
                    audio_outputs_count: 1,
                    in_events: ptr::null(), // TODO: 事件
                    out_events: ptr::null(),
                };

                process_fn(p, &process_data);

                // 重新交错回去
                for i in 0..frames {
                    let si = i * 2;
                    buffer.samples[si] = out_l[i];
                    buffer.samples[si + 1] = out_r[i];
                }
            }
        } else {
            // 未加载插件 -> 输出静音
            for s in buffer.samples.iter_mut() {
                *s = 0.0;
            }
        }
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0
    }

    fn set_param(&mut self, _id: u32, _value: f32) {}
}
