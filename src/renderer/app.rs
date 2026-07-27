use std::ffi::CString;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const REQUIRED_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct AppInfo
{
    pub app_name: CString,
    pub app_version: u32,
    pub api_version: u32
}