use std::ffi::CString;

use Singularity::renderer::app::AppInfo;
use Singularity::renderer::instance::Instance;

fn main()
{
    let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan") };

    let app_info = AppInfo {
        app_name: CString::new("Singularity").unwrap(),
        app_version: ash::vk::make_api_version(0, 0, 1, 0),
        api_version: ash::vk::make_api_version(0, 1, 4, 0),
    };

    let _instance = Instance::new(&entry, app_info)
        .expect("Failed to create Vulkan instance");
}