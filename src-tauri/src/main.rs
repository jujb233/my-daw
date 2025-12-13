// 在 Windows 发布版中防止额外的控制台窗口，勿移除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
        // 修复 Linux 上出现的 “Error 71 (Protocol error) dispatching to Wayland display” 问题
        #[cfg(target_os = "linux")]
        unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }

        my_daw_lib::run()
}
