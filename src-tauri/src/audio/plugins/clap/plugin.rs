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

// CLAP 主机（Host）的最小实现
unsafe extern "C" fn host_get_extension(
    _host: *const clap_host,
    _extension_id: *const std::os::raw::c_char,
) -> *const std::ffi::c_void {
    ptr::null()
}

unsafe extern "C" fn host_request_restart(_host: *const clap_host) {}
unsafe extern "C" fn host_request_process(_host: *const clap_host) {}
unsafe extern "C" fn host_request_callback(_host: *const clap_host) {}

static HOST: clap_host = clap_host {
    clap_version: clap_sys::version::CLAP_VERSION,
    host_data: ptr::null_mut(),
    name: b"MyDAW\0".as_ptr() as *const _,
    vendor: b"MyDAW\0".as_ptr() as *const _,
    url: b"\0".as_ptr() as *const _,
    version: b"0.1.0\0".as_ptr() as *const _,
    get_extension: Some(host_get_extension),
    request_restart: Some(host_request_restart),
    request_process: Some(host_request_process),
    request_callback: Some(host_request_callback),
};

pub struct ClapPlugin {
    _library: Arc<Library>, // 保持库加载（防止被卸载）
    plugin: *const clap_plugin,
    info: PluginInfo,
    #[allow(dead_code)]
    io_config: IOConfig,
    params: Vec<PluginParameter>,
}

unsafe impl Send for ClapPlugin {}
unsafe impl Sync for ClapPlugin {}

impl ClapPlugin {
    pub unsafe fn new(path: &str) -> Result<Self, String> { unsafe {
        let lib = Library::new(path).map_err(|e| e.to_string())?;
        let lib = Arc::new(lib);

        let entry_fn: Symbol<
            unsafe extern "C" fn(path: *const std::os::raw::c_char) -> *const clap_plugin_entry,
        > = lib.get(b"clap_entry").map_err(|e| e.to_string())?;

        let c_path = CString::new(path).unwrap();
        let entry = entry_fn(c_path.as_ptr());

        if entry.is_null() {
            return Err("Failed to get clap_entry".to_string());
        }

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

        let plugin_id = (*descriptor).id;
        let create_plugin = (*factory).create_plugin.ok_or("No create_plugin")?;

        let plugin = create_plugin(factory, &HOST, plugin_id);
        if plugin.is_null() {
            return Err("Failed to create plugin instance".to_string());
        }

        let init_plugin = (*plugin).init.ok_or("No plugin init")?;
        if !init_plugin(plugin) {
            return Err("Failed to initialize plugin instance".to_string());
        }

        let activate = (*plugin).activate.ok_or("No activate")?;
        // 使用采样率和块大小进行激活
        // TODO: 获取真实的采样率
        if !activate(plugin, 44100.0, 32, 4096) {
            return Err("Failed to activate plugin".to_string());
        }

        let name = CStr::from_ptr((*descriptor).name)
            .to_string_lossy()
            .into_owned();
        let vendor = CStr::from_ptr((*descriptor).vendor)
            .to_string_lossy()
            .into_owned();
        let unique_id = CStr::from_ptr(plugin_id).to_string_lossy().into_owned();

        // TODO: 扫描参数
        let params = Vec::new();

        Ok(Self {
            _library: lib,
            plugin,
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
        })
    }}
}

impl Drop for ClapPlugin {
    fn drop(&mut self) {
        unsafe {
            if let Some(deactivate) = (*self.plugin).deactivate {
                deactivate(self.plugin);
            }
            if let Some(destroy) = (*self.plugin).destroy {
                destroy(self.plugin);
            }
        }
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
        unsafe {
            let process_fn = (*self.plugin).process.unwrap();

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

            let mut out_l = vec![0.0f32; buffer.samples.len() / 2];
            let mut out_r = vec![0.0f32; buffer.samples.len() / 2];

            // 去交错输入（如果有）- 暂时跳过（合成器）

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
                frames_count: (buffer.samples.len() / 2) as u32,
                transport: ptr::null(),
                audio_inputs: ptr::null_mut(), // 无输入
                audio_outputs: &mut audio_out,
                audio_inputs_count: 0,
                audio_outputs_count: 1,
                in_events: ptr::null(), // TODO: 事件
                out_events: ptr::null(),
            };

            process_fn(self.plugin, &process_data);

            // 重新交错回去
            for i in 0..out_l.len() {
                if i * 2 + 1 < buffer.samples.len() {
                    buffer.samples[i * 2] = out_l[i];
                    buffer.samples[i * 2 + 1] = out_r[i];
                }
            }
        }
    }

    fn get_param(&self, _id: u32) -> f32 {
        0.0
    }

    fn set_param(&mut self, _id: u32, _value: f32) {}
}
