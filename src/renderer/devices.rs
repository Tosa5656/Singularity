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

pub struct Device
{
    device: ash::Device,
    graphics_queue: vk::Queue
}
impl Device
{
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Result<Self, vk::Result>
    {
        let queue_family_index = Self::find_queue_families(instance, physical_device).unwrap();
        let queue_priorities = [1.0f32];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)];

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features);

        let device = unsafe {
            instance.instance
                .create_device(physical_device.physical_device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self { device, graphics_queue })
    }

    fn find_queue_families(instance: &Instance, physical_device: &PhysicalDevice) -> Option<u32>
    {
        let props = unsafe { instance.instance.get_physical_device_queue_family_properties(physical_device.physical_device) };
        props
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(index, _)| index as u32)
    }
}
impl Drop for Device
{
    fn drop(&mut self)
    {
        unsafe { self.device.destroy_device(None); }
    }
}
