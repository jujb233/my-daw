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

// Minimal Host Implementation
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
    _library: Arc<Library>, // Keep library loaded
    plugin: *const clap_plugin,
    info: PluginInfo,
    #[allow(dead_code)]
    io_config: IOConfig,
    params: Vec<PluginParameter>,
}

unsafe impl Send for ClapPlugin {}
unsafe impl Sync for ClapPlugin {}

impl ClapPlugin {
    pub unsafe fn new(path: &str) -> Result<Self, String> {
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

        // Just take the first plugin for now
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
        // Activate with sample rate and block size
        // TODO: Get real sample rate
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

        // TODO: Scan parameters
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
            }, // Assume stereo out for now
            params,
        })
    }
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

            // Prepare Audio Buffers
            // CLAP expects an array of pointers to f32 arrays (for non-interleaved)
            // Our AudioBuffer is interleaved or non-interleaved?
            // `AudioBuffer` struct has `samples: &mut [f32]`. This is usually interleaved in our simple engine.
            // But CLAP usually wants non-interleaved.
            // If our engine is interleaved, we need to de-interleave.
            // For now, let's assume we only support 1 channel or we hack it.
            // Wait, `AudioBuffer` in `plugin.rs` has `channels`.

            // We need scratch buffers for de-interleaving if we are interleaved.
            // Let's assume for a moment we are just passing pointers.
            // But we can't easily de-interleave in the audio thread without allocation if we didn't prepare.

            // For this prototype, let's assume Stereo and we de-interleave into stack arrays if small enough,
            // or just process silence if we can't.

            // Actually, let's just implement a dummy process for now to prove it loads.
            // Real audio processing requires buffer management.

            let mut out_l = vec![0.0f32; buffer.samples.len() / 2];
            let mut out_r = vec![0.0f32; buffer.samples.len() / 2];

            // De-interleave input (if any) - skipping for now (synth)

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
                audio_inputs: ptr::null_mut(), // No inputs
                audio_outputs: &mut audio_out,
                audio_inputs_count: 0,
                audio_outputs_count: 1,
                in_events: ptr::null(), // TODO: Events
                out_events: ptr::null(),
            };

            process_fn(self.plugin, &process_data);

            // Interleave back
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
