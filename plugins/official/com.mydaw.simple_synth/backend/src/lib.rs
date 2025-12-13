use serde::{Deserialize, Serialize};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    id: String,
    name: String,
}

struct SimpleSynth {
    frequency: f32,
    sample_rate: f32,
}

#[no_mangle]
pub extern "C" fn create_plugin(sample_rate: f32) -> *mut c_void {
    let plugin = Box::new(SimpleSynth {
        frequency: 440.0,
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
        Box::from_raw(ptr as *mut SimpleSynth);
    }
}

#[no_mangle]
pub extern "C" fn plugin_info_json() -> *mut c_char {
    let info = PluginInfo {
        id: "com.mydaw.simplesynth".to_string(),
        name: "Simple Synth".to_string(),
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
    let synth = unsafe { &mut *(ptr as *mut SimpleSynth) };
    // write a sine for testing
    let two_pi = std::f32::consts::PI * 2.0;
    for frame in 0..frames {
        let t = frame as f32 / synth.sample_rate;
        let sample = (synth.frequency * two_pi * t).sin() * 0.25;
        for ch in 0..channels {
            unsafe {
                *out_ptr.add(frame * channels + ch) = sample;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_param(ptr: *mut c_void, id: u32, value: f32) {
    if ptr.is_null() {
        return;
    }
    let synth = unsafe { &mut *(ptr as *mut SimpleSynth) };
    if id == 0 {
        synth.frequency = value;
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_param(ptr: *mut c_void, id: u32) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    let synth = unsafe { &mut *(ptr as *mut SimpleSynth) };
    if id == 0 {
        synth.frequency
    } else {
        0.0
    }
}
