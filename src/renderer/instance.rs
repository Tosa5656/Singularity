use ash::vk;
use std::ffi::CString;
use super::app::*;

pub struct Instance
{
    pub instance: ash::Instance
}
impl Instance
{
    pub fn new(entry: &ash::Entry, app_info: AppInfo) -> Result<Self, vk::Result>
    {
        let c_engine_name = CString::new("Singularity Engine").unwrap();

        let application_info = vk::ApplicationInfo::default()
            .application_name(app_info.app_name.as_c_str())
            .application_version(app_info.app_version)
            .engine_name(c_engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(app_info.api_version);

        let extension_names = required_extension_names();

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

        Ok(Self { instance })
    }
}