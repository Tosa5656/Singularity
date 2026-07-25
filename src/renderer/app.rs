use std::ffi::CString;
use winit::raw_window_handle::HasDisplayHandle;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const REQUIRED_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct AppInfo
{
    pub app_name: CString,
    pub app_version: u32,
    pub api_version: u32
}

pub fn required_extension_names(window: &impl HasDisplayHandle) -> Vec<*const std::os::raw::c_char>
{
    let raw_display = window.display_handle().unwrap().as_raw();
    let raw_extensions = ash_window::enumerate_required_extensions(raw_display).unwrap();
    raw_extensions.iter().copied().collect()
}
