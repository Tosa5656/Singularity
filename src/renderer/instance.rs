use ash::vk;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use super::app::*;

pub struct Instance
{
    pub instance: ash::Instance,
    pub debug_report_callback: Option<(ash::ext::debug_report::Instance, vk::DebugReportCallbackEXT)>
}
impl Instance
{
    pub fn new(entry: &ash::Entry, app_info: AppInfo, window: &winit::window::Window) -> Result<Self, vk::Result>
    {
        let c_engine_name = CString::new("Singularity Engine").unwrap();

        let application_info = vk::ApplicationInfo::default()
            .application_name(app_info.app_name.as_c_str())
            .application_version(app_info.app_version)
            .engine_name(c_engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(app_info.api_version);

        let mut extension_names = required_extension_names(window);
        if ENABLE_VALIDATION_LAYERS
        {
            extension_names.push(ash::vk::EXT_DEBUG_REPORT_NAME.as_ptr());
        }

        let layer_names = REQUIRED_LAYERS
            .iter()
            .map(|name| CString::new(*name).expect("Failed to build CString"))
            .collect::<Vec<_>>();
        let layer_names_ptrs = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names);
        if ENABLE_VALIDATION_LAYERS
        {
            Self::check_validation_layer_support(entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptrs);
        }

        let instance = unsafe { entry.create_instance(&instance_create_info, None)? };
        let debug_report_callback = Self::setup_debug_messenger(entry, &instance);

        Ok(Self { instance, debug_report_callback })
    }

    unsafe extern "system" fn vulkan_debug_callback(
        flag: vk::DebugReportFlagsEXT,
        typ: vk::DebugReportObjectTypeEXT,
        _: u64,
        _: usize,
        _: i32,
        _: *const c_char,
        p_message: *const c_char,
        _: *mut c_void,
        ) -> u32
    {
        let message = unsafe { CStr::from_ptr(p_message) };
        if flag == vk::DebugReportFlagsEXT::DEBUG
        {
            log::debug!("{:?} - {:?}", typ, message);
        }
        else if flag == vk::DebugReportFlagsEXT::INFORMATION
        {
            log::info!("{:?} - {:?}", typ, message);
        }
        else if flag == vk::DebugReportFlagsEXT::WARNING
        {
            log::warn!("{:?} - {:?}", typ, message);
        }
        else if flag == vk::DebugReportFlagsEXT::PERFORMANCE_WARNING
        {
            log::warn!("{:?} - {:?}", typ, message);
        }
        else
        {
            log::error!("{:?} - {:?}", typ, message);
        }
        vk::FALSE
    }

    fn check_validation_layer_support(entry: &ash::Entry)
    {
        for required in REQUIRED_LAYERS.iter() {
            let found = unsafe {
                entry
                    .enumerate_instance_layer_properties()
                    .unwrap()
                    .iter()
                    .any(|layer| {
                        let name = CStr::from_ptr(layer.layer_name.as_ptr());
                        let name = name.to_str().expect("Failed to get layer name pointer");
                        required == &name
                    })
            };

            if !found {
                panic!("Validation layer not supported: {}", required);
            }
        }
    }

    fn setup_debug_messenger(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> Option<(ash::ext::debug_report::Instance, vk::DebugReportCallbackEXT)> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }
        let create_info = vk::DebugReportCallbackCreateInfoEXT::default()
            .flags(
                vk::DebugReportFlagsEXT::INFORMATION
                    | vk::DebugReportFlagsEXT::WARNING
                    | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING
                    | vk::DebugReportFlagsEXT::ERROR
                    | vk::DebugReportFlagsEXT::DEBUG,
            )
            .pfn_callback(Some(Self::vulkan_debug_callback));
        let debug_report = ash::ext::debug_report::Instance::new(entry, instance);
        let debug_report_callback = unsafe {
            debug_report
                .create_debug_report_callback(&create_info, None)
                .unwrap()
        };
        Some((debug_report, debug_report_callback))
    }
}
