use libc::c_void;
use libloading::{Library, Symbol};
use std::ffi::{c_char, CString};
use std::fs;
use std::os::raw::c_float;
use std::path::PathBuf;
use std::ptr;

#[repr(C)]
struct ChildPlugin {
    lib: Library,
    path: String,
    inst: *mut c_void,
    destroy_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void)>,
    process_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize)>,
    set_param_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void, u32, f32)>,
    get_param_fn: Symbol<'static, unsafe extern "C" fn(*mut c_void, u32) -> f32>,
}

struct ContainerInstance {
    children: Vec<ChildPlugin>,
    child_paths: Vec<String>,
    sample_rate: f32,
}

unsafe fn load_child(lib_path: &str, sample_rate: f32) -> Result<ChildPlugin, String> {
    let lib = Library::new(lib_path).map_err(|e| e.to_string())?;
    // Safety: extend lifetime to 'static by leaking library into Box
    let lib_box = Box::new(lib);
    let lib_ref: &'static Library = Box::leak(lib_box);

    let create: Symbol<unsafe extern "C" fn(f32) -> *mut c_void> =
        lib_ref.get(b"create_plugin").map_err(|e| e.to_string())?;
    let destroy: Symbol<unsafe extern "C" fn(*mut c_void)> =
        lib_ref.get(b"destroy_plugin").map_err(|e| e.to_string())?;
    let process: Symbol<unsafe extern "C" fn(*mut c_void, *mut f32, usize, usize)> =
        lib_ref.get(b"plugin_process").map_err(|e| e.to_string())?;
    let set_param: Symbol<unsafe extern "C" fn(*mut c_void, u32, f32)> = lib_ref
        .get(b"plugin_set_param")
        .map_err(|e| e.to_string())?;
    let get_param: Symbol<unsafe extern "C" fn(*mut c_void, u32) -> f32> = lib_ref
        .get(b"plugin_get_param")
        .map_err(|e| e.to_string())?;

    let inst = unsafe { create(sample_rate) };
    Ok(ChildPlugin {
        lib: unsafe { std::mem::transmute_copy(&lib_ref) },
        path: lib_path.to_string(),
        inst,
        destroy_fn: unsafe { std::mem::transmute(destroy) },
        process_fn: unsafe { std::mem::transmute(process) },
        set_param_fn: unsafe { std::mem::transmute(set_param) },
        get_param_fn: unsafe { std::mem::transmute(get_param) },
    })
}

// Try to determine the path to this shared library using dladdr on a local symbol.
fn get_own_folder() -> Option<PathBuf> {
    unsafe {
        extern "C" fn anchor() {}
        let addr = anchor as *const ();
        let mut info: libc::Dl_info = std::mem::zeroed();
        if libc::dladdr(addr as *const c_void, &mut info) == 0 {
            return None;
        }
        if info.dli_fname.is_null() {
            return None;
        }
        let cstr = std::ffi::CStr::from_ptr(info.dli_fname);
        if let Ok(s) = cstr.to_str() {
            let p = PathBuf::from(s);
            if let Some(dir) = p.parent() {
                return Some(dir.to_path_buf());
            }
        }
        None
    }
}

#[no_mangle]
pub extern "C" fn create_plugin(sample_rate: f32) -> *mut c_void {
    // Find manifest.lua next to this library
    let folder = match get_own_folder() {
        Some(f) => f,
        None => return ptr::null_mut(),
    };
    let manifest = folder.join("../manifest.lua");
    let mut children = Vec::new();
    let mut child_paths: Vec<String> = Vec::new();
    if manifest.exists() {
        if let Ok(content) = fs::read_to_string(&manifest) {
            // Minimal parsing: look for backend local paths under children table
            // We'll use lua to parse if available
            if let Ok(lua) = mlua::Lua::new().load(&content).eval::<mlua::Table>() {
                // manifest may contain a 'children' table
                if let Ok(children_tbl) = lua.get::<_, mlua::Table>("children") {
                    for pair in children_tbl.sequence_values::<mlua::Table>() {
                        if let Ok(tbl) = pair {
                            if let Ok(backend) = tbl.get::<_, mlua::Table>("backend") {
                                if let Ok(t) = backend.get::<_, String>("type") {
                                    if t == "local" {
                                        if let Ok(p) = backend.get::<_, String>("path") {
                                            // resolve relative to plugin folder
                                            let child_path = folder.join("../").join(p);
                                            if let Some(s) = child_path.to_str() {
                                                match unsafe { load_child(s, sample_rate as f32) } {
                                                    Ok(child) => {
                                                        child_paths.push(s.to_string());
                                                        children.push(child)
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed load child {}: {}", s, e)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let inst = Box::new(ContainerInstance {
        children,
        child_paths,
        sample_rate: sample_rate as f32,
    });
    Box::into_raw(inst) as *mut c_void
}

#[no_mangle]
pub extern "C" fn destroy_plugin(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let inst: Box<ContainerInstance> = Box::from_raw(ptr as *mut ContainerInstance);
        // Drop children: call destroy then drop library
        for mut child in inst.children {
            (child.destroy_fn)(child.inst);
            // Library will be dropped when child goes out of scope
            let _ = child;
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_state(ptr: *mut c_void, out_len: *mut usize) -> *mut u8 {
    if ptr.is_null() {
        if !out_len.is_null() {
            unsafe { *out_len = 0 };
        }
        return ptr::null_mut();
    }
    unsafe {
        let inst = &*(ptr as *mut ContainerInstance);
        let state = serde_json::json!({"children": inst.child_paths});
        if let Ok(bytes) = serde_json::to_vec(&state) {
            let len = bytes.len();
            if len == 0 {
                if !out_len.is_null() {
                    *out_len = 0;
                }
                return ptr::null_mut();
            }
            let buf = libc::malloc(len) as *mut u8;
            if buf.is_null() {
                if !out_len.is_null() {
                    *out_len = 0;
                }
                return ptr::null_mut();
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
            if !out_len.is_null() {
                *out_len = len;
            }
            return buf;
        }
    }
    if !out_len.is_null() {
        unsafe { *out_len = 0 }
    }
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn plugin_free_state_blob(ptr: *mut u8, _len: usize) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        libc::free(ptr as *mut c_void);
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_state(ptr: *mut c_void, data: *const u8, len: usize) {
    if ptr.is_null() || data.is_null() || len == 0 {
        return;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(slice) {
        if let Some(children_arr) = v.get("children").and_then(|c| c.as_array()) {
            unsafe {
                let inst = &mut *(ptr as *mut ContainerInstance);
                // destroy existing
                for mut child in inst.children.drain(..) {
                    (child.destroy_fn)(child.inst);
                }
                inst.child_paths.clear();
                // reload according to listed paths
                let folder = match get_own_folder() {
                    Some(f) => f,
                    None => return,
                };
                for item in children_arr {
                    if let Some(s) = item.as_str() {
                        // resolve relative to manifest folder
                        let child_path = folder.join("../").join(s);
                        if let Some(sp) = child_path.to_str() {
                            match unsafe { load_child(sp, inst.sample_rate) } {
                                Ok(child) => {
                                    inst.child_paths.push(sp.to_string());
                                    inst.children.push(child);
                                }
                                Err(e) => eprintln!("Failed load child {}: {}", sp, e),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_process(
    ptr: *mut c_void,
    samples: *mut f32,
    frames: usize,
    channels: usize,
) {
    if ptr.is_null() || samples.is_null() {
        return;
    }
    let buf = unsafe { std::slice::from_raw_parts_mut(samples, frames * channels) };
    unsafe {
        let inst = &mut *(ptr as *mut ContainerInstance);
        // Chain processing through children sequentially
        for child in &mut inst.children {
            (child.process_fn)(child.inst, buf.as_mut_ptr(), frames, channels);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_set_param(ptr: *mut c_void, id: u32, value: f32) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let inst = &mut *(ptr as *mut ContainerInstance);
        // Apply to all children as a simple policy
        for child in &mut inst.children {
            (child.set_param_fn)(child.inst, id, value);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_get_param(ptr: *mut c_void, id: u32) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    unsafe {
        let inst = &mut *(ptr as *mut ContainerInstance);
        if let Some(child) = inst.children.first() {
            return (child.get_param_fn)(child.inst, id);
        }
    }
    0.0
}

#[no_mangle]
pub extern "C" fn plugin_info_json() -> *mut c_char {
    let info = serde_json::json!({
        "id": "com.mydaw.container",
        "name": "Plugin Container",
        "children": []
    });
    let s = info.to_string();
    let c = CString::new(s).unwrap();
    c.into_raw()
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
