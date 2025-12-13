use serde::{Deserialize, Serialize};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    id: String,
    name: String,
}

struct WaveGenerator {
    phase: f32,
    sample_rate: f32,
}

#[no_mangle]
pub extern "C" fn create_plugin(sample_rate: f32) -> *mut c_void {
    let plugin = Box::new(WaveGenerator {
        phase: 0.0,
        sample_rate,
    });
    Box::into_raw(plugin) as *mut c_void
}

#[no_mangle]
pub extern "C" fn destroy_plugin(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr as *mut WaveGenerator);
    }
}

#[no_mangle]
pub extern "C" fn plugin_info_json() -> *mut c_char {
    let info = PluginInfo {
        id: "com.mydaw.wavegenerator".to_string(),
        name: "Wave Generator".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap_or_default();
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        CString::from_raw(s);
    }
}

#[no_mangle]
pub extern "C" fn plugin_process(
    ptr: *mut c_void,
    out_ptr: *mut f32,
    frames: usize,
    channels: usize,
) {
    if ptr.is_null() {
        return;
    }
    let gen = unsafe { &mut *(ptr as *mut WaveGenerator) };
    let two_pi = std::f32::consts::PI * 2.0;
    let freq = 220.0;
    for frame in 0..frames {
        let sample = (gen.phase * two_pi).sin() * 0.2;
        for ch in 0..channels {
            unsafe {
                *out_ptr.add(frame * channels + ch) = sample;
            }
        }
        gen.phase += freq / gen.sample_rate;
        if gen.phase >= 1.0 {
            gen.phase -= 1.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_param(_ptr: *mut c_void, _id: u32, _value: f32) {}

#[no_mangle]
pub extern "C" fn plugin_get_param(_ptr: *mut c_void, _id: u32) -> f32 {
    0.0
}
