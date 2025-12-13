use serde::{Deserialize, Serialize};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    id: String,
    name: String,
}

struct LevelMeter {
    peak: f32,
}

#[no_mangle]
pub extern "C" fn create_plugin(_sample_rate: f32) -> *mut c_void {
    let plugin = Box::new(LevelMeter { peak: 0.0 });
    Box::into_raw(plugin) as *mut c_void
}

#[no_mangle]
pub extern "C" fn destroy_plugin(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr as *mut LevelMeter);
    }
}

#[no_mangle]
pub extern "C" fn plugin_info_json() -> *mut c_char {
    let info = PluginInfo {
        id: "com.mydaw.levelmeter".to_string(),
        name: "Level Meter".to_string(),
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
pub extern "C" fn plugin_process(ptr: *mut c_void, buf: *mut f32, frames: usize, channels: usize) {
    if ptr.is_null() {
        return;
    }
    let meter = unsafe { &mut *(ptr as *mut LevelMeter) };
    let mut peak: f32 = 0.0;
    for frame in 0..frames {
        for ch in 0..channels {
            let sample = unsafe { *buf.add(frame * channels + ch) };
            peak = peak.max(sample.abs());
        }
    }
    meter.peak = peak;
}

#[no_mangle]
pub extern "C" fn plugin_get_peak(ptr: *mut c_void) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    let meter = unsafe { &mut *(ptr as *mut LevelMeter) };
    meter.peak
}
