use ash::vk;
use ash::khr::swapchain;

use crate::renderer::devices::{Device, PhysicalDevice};
use crate::renderer::instance::Instance;
use crate::renderer::surface::Surface;

pub(crate) struct SwapChainSupportDetails
{
    pub(crate) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<vk::SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<vk::PresentModeKHR>,
}
impl SwapChainSupportDetails
{
    pub(crate) fn new(physical_device: &PhysicalDevice, surface: &Surface) -> Self
    {
        let capabilities = unsafe {
            surface.surface_fn
                .get_physical_device_surface_capabilities(physical_device.physical_device, surface.surface_khr)
                .unwrap()
        };

        let formats = unsafe {
            surface.surface_fn
                .get_physical_device_surface_formats(physical_device.physical_device, surface.surface_khr)
                .unwrap()
        };

        let present_modes = unsafe {
            surface.surface_fn
                .get_physical_device_surface_present_modes(physical_device.physical_device, surface.surface_khr)
                .unwrap()
        };

        Self { capabilities, formats, present_modes }
    }
}

pub struct SwapChain
{
    swapchain_loader: swapchain::Device,
    swapchain_khr: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_format: vk::Format,
    extent: vk::Extent2D,
}
impl SwapChain
{
    pub fn name() -> &'static std::ffi::CStr
    {
        swapchain::NAME
    }

    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        surface: &Surface,
        width: u32,
        height: u32,
    ) -> Result<Self, vk::Result>
    {
        let details = SwapChainSupportDetails::new(physical_device, surface);
        let format = Self::choose_surface_format(&details.formats);
        let present_mode = Self::choose_surface_present_mode(&details.present_modes);
        let extent = Self::choose_extent(details.capabilities, width, height);

        let image_count =
        {
            let max = details.capabilities.max_image_count;
            let mut preferred = details.capabilities.min_image_count + 1;
            if max > 0 && preferred > max
            {
                preferred = max;
            }
            preferred
        };

        log::debug!(
            "Creating swapchain.\n\tFormat: {:?}\n\tColorSpace: {:?}\n\tPresentMode: {:?}\n\tExtent: {:?}\n\tImageCount: {}",
            format.format,
            format.color_space,
            present_mode,
            extent,
            image_count,
        );

        let concurrent = device.graphics_family_index != device.present_family_index;
        let families_indices = [device.graphics_family_index, device.present_family_index];

        let create_info = vk::SwapchainCreateInfoKHR
        {
            surface: surface.surface_khr,
            min_image_count: image_count,
            image_format: format.format,
            image_color_space: format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: if concurrent { vk::SharingMode::CONCURRENT } else { vk::SharingMode::EXCLUSIVE },
            queue_family_index_count: if concurrent { 2 } else { 0 },
            p_queue_family_indices: if concurrent { families_indices.as_ptr() } else { std::ptr::null() },
            pre_transform: details.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        let swapchain_loader = swapchain::Device::new(&instance.instance, &device.device);
        let swapchain_khr = unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain_khr).unwrap() };
        Ok(Self { swapchain_loader, swapchain_khr, images, image_format: format.format, extent })
    }

    pub fn image_format(&self) -> vk::Format
    {
        self.image_format
    }

    pub fn extent(&self) -> vk::Extent2D
    {
        self.extent
    }

    pub fn images(&self) -> &[vk::Image]
    {
        &self.images
    }

    fn choose_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR
    {
        if available_formats.len() == 1 && available_formats[0].format == vk::Format::UNDEFINED {
            return vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            };
        }

        *available_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&available_formats[0])
    }

    fn choose_surface_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR
    {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX)
        {
            vk::PresentModeKHR::MAILBOX
        }
        else if available_present_modes.contains(&vk::PresentModeKHR::FIFO)
        {
            vk::PresentModeKHR::FIFO
        }
        else
        {
            vk::PresentModeKHR::IMMEDIATE
        }
    }

    fn choose_extent(capabilities: vk::SurfaceCapabilitiesKHR, width: u32, height: u32) -> vk::Extent2D
    {
        if capabilities.current_extent.width != std::u32::MAX
        {
            return capabilities.current_extent;
        }

        let min = capabilities.min_image_extent;
        let max = capabilities.max_image_extent;
        let width = width.min(max.width).max(min.width);
        let height = height.min(max.height).max(min.height);
        vk::Extent2D { width, height }
    }
}
impl Drop for SwapChain
{
    fn drop(&mut self)
    {
        unsafe { self.swapchain_loader.destroy_swapchain(self.swapchain_khr, None); }
    }
}
