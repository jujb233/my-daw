// Windows 发布版：隐藏额外的控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
        // Linux: 禁用 WebKit DMA-BUF 渲染以规避 Wayland 的 Error 71 问题
        #[cfg(target_os = "linux")]
        unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }

        my_daw_lib::run()
}
