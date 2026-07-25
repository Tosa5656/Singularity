use std::ffi::CString;

pub struct AppInfo
{
    pub app_name: CString,
    pub app_version: u32,
    pub api_version: u32
}

pub fn required_extension_names() -> Vec<*const i8>
{
    vec![
        ash::vk::KHR_SURFACE_NAME.as_ptr(),
        ash::vk::KHR_WIN32_SURFACE_NAME.as_ptr(),
    ]
}
