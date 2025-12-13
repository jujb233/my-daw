use serde::{Deserialize, Serialize};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    id: String,
    name: String,
}

struct GainFader {
    gain_db: f32,
}

#[no_mangle]
pub extern "C" fn create_plugin(_sample_rate: f32) -> *mut c_void {
    let plugin = Box::new(GainFader { gain_db: 0.0 });
    Box::into_raw(plugin) as *mut c_void
}

#[no_mangle]
pub extern "C" fn destroy_plugin(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr as *mut GainFader);
    }
}

#[no_mangle]
pub extern "C" fn plugin_info_json() -> *mut c_char {
    let info = PluginInfo {
        id: "com.mydaw.gainfader".to_string(),
        name: "Gain Fader".to_string(),
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
    io_ptr: *mut f32,
    frames: usize,
    channels: usize,
) {
    if ptr.is_null() {
        return;
    }
    let gain = unsafe { &mut *(ptr as *mut GainFader) };
    let linear = 10f32.powf(gain.gain_db / 20.0);
    for frame in 0..frames {
        for ch in 0..channels {
            unsafe {
                *io_ptr.add(frame * channels + ch) *= linear;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_param(ptr: *mut c_void, id: u32, value: f32) {
    if ptr.is_null() {
        return;
    }
    let gain = unsafe { &mut *(ptr as *mut GainFader) };
    if id == 0 {
        gain.gain_db = value;
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_param(ptr: *mut c_void, id: u32) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    let gain = unsafe { &mut *(ptr as *mut GainFader) };
    if id == 0 {
        gain.gain_db
    } else {
        0.0
    }
}
