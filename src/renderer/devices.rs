use ash::vk;
use std::ffi::CStr;

use crate::renderer::instance::Instance;
use crate::renderer::surface::Surface;
use crate::renderer::swapchain::{SwapChain, SwapChainSupportDetails};

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
        let extention_support = Self::check_extension_support(instance, device);

        let physical_device_wrapper = PhysicalDevice { physical_device: device };
        let details = SwapChainSupportDetails::new(&physical_device_wrapper, surface);
        let is_swapchain_adequate = !details.formats.is_empty() && !details.present_modes.is_empty();

        graphics.is_some() && present.is_some() && extention_support && is_swapchain_adequate
    }

    pub fn check_extension_support(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool
    {
        let required_extentions = [SwapChain::name()];

        let extension_props = unsafe
        {
            instance.instance
                .enumerate_device_extension_properties(physical_device)
                .unwrap()
        };

        for required in required_extentions.iter()
        {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }

        true
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
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub graphics_family_index: u32,
    pub present_family_index: u32,
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

        let device_extensions = [SwapChain::name()];
        let device_extensions_ptrs = device_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions_ptrs)
            .enabled_features(&device_features);

        let device = unsafe {
            instance.instance
                .create_device(physical_device.physical_device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };

        Ok(Self { device, graphics_queue, present_queue, graphics_family_index, present_family_index })
    }
}
impl Drop for Device
{
    fn drop(&mut self)
    {
        unsafe { self.device.destroy_device(None); }
    }
}
