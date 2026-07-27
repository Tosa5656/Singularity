use ash::vk;
use std::ffi::CStr;

use crate::renderer::instance::Instance;
use crate::renderer::surface::Surface;

pub struct PhysicalDevice
{
    pub physical_device: vk::PhysicalDevice
}
impl PhysicalDevice
{
    pub fn new(instance: &Instance, surface: &Surface) -> Result<Self, vk::Result>
    {
        let physical_device = Self::pick(instance, surface);

        Ok(Self { physical_device })
    }

    fn pick(instance: &Instance, surface: &Surface) -> vk::PhysicalDevice
    {
        let devices = unsafe { instance.instance.enumerate_physical_devices().unwrap() };
        let device = devices
            .into_iter()
            .find(|device| Self::is_suitable(instance, surface, *device))
            .expect("No suitable physical device.");

        let props = unsafe { instance.instance.get_physical_device_properties(device) };
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });
        device
    }

    fn is_suitable(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice) -> bool
    {
        let (graphics, present) = Self::find_queue_families(instance, surface, device);
        graphics.is_some() && present.is_some()
    }

    pub(crate) fn find_queue_families(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice) -> (Option<u32>, Option<u32>)
    {
        let mut graphics = None;
        let mut present = None;

        let props = unsafe { instance.instance.get_physical_device_queue_family_properties(device) };

        for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate()
        {
            let index = index as u32;

            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none()
            {
                graphics = Some(index);
            }

            let present_support = surface.get_physical_device_support(device, index);
            if present_support && present.is_none()
            {
                present = Some(index);
            }

            if graphics.is_some() && present.is_some()
            {
                break;
            }
        }

        (graphics, present)
    }
}

pub struct Device
{
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue
}
impl Device
{
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice, surface: &Surface) -> Result<Self, vk::Result>
    {
        let (graphics_family_index, present_family_index) = PhysicalDevice::find_queue_families(instance, surface, physical_device.physical_device);
        let graphics_family_index = graphics_family_index.unwrap();
        let present_family_index = present_family_index.unwrap();

        let queue_priorities = [1.0f32];

        let queue_create_infos =
        {
            let mut indices = vec![graphics_family_index, present_family_index];
            indices.dedup();

            indices
                .iter()
                .map(|index| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*index)
                        .queue_priorities(&queue_priorities)
                })
                .collect::<Vec<_>>()
        };

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features);

        let device = unsafe {
            instance.instance
                .create_device(physical_device.physical_device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };

        Ok(Self { device, graphics_queue, present_queue })
    }
}
impl Drop for Device
{
    fn drop(&mut self)
    {
        unsafe { self.device.destroy_device(None); }
    }
}
