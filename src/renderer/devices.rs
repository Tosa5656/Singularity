use ash::vk;
use std::ffi::CStr;

use crate::renderer::instance::Instance;

pub struct PhysicalDevice
{
    pub physical_device: vk::PhysicalDevice
}
impl PhysicalDevice
{
    pub fn new(instance: &Instance) -> Result<Self, vk::Result>
    {
        let physical_device = Self::pick(instance);

        Ok(Self { physical_device })
    }

    fn pick(instance: &Instance) -> vk::PhysicalDevice
    {
        let devices = unsafe { instance.instance.enumerate_physical_devices().unwrap() };
        let device = devices
            .into_iter()
            .find(|device| Self::is_suitable(instance, *device))
            .expect("No suitable physical device.");

        let props = unsafe { instance.instance.get_physical_device_properties(device) };
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });
        device
    }

    fn is_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool
    {
        Self::find_queue_families(instance, device).is_some()
    }

    fn find_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<usize>
    {
        let props = unsafe { instance.instance.get_physical_device_queue_family_properties(device) };
        props
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(index, _)| index)
    }
}