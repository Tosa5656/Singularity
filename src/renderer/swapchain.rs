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
        Self::query(&surface.surface_fn, surface.surface_khr, physical_device.physical_device)
    }

    fn query(
        surface_fn: &ash::khr::surface::Instance,
        surface_khr: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Self
    {
        let capabilities = unsafe
        {
            surface_fn
                .get_physical_device_surface_capabilities(physical_device, surface_khr)
                .unwrap()
        };

        let formats = unsafe
        {
            surface_fn
                .get_physical_device_surface_formats(physical_device, surface_khr)
                .unwrap()
        };

        let present_modes = unsafe
        {
            surface_fn
                .get_physical_device_surface_present_modes(physical_device, surface_khr)
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
    pub image_views: Vec<vk::ImageView>,
    device: ash::Device,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    graphics_family_index: u32,
    present_family_index: u32,
    surface_fn: ash::khr::surface::Instance,
    surface_khr: vk::SurfaceKHR,
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
        Self::create(
            &instance.instance,
            &device.device,
            device.graphics_family_index,
            device.present_family_index,
            physical_device.physical_device,
            &surface.surface_fn,
            surface.surface_khr,
            width,
            height,
        )
    }

    pub fn recreate(&mut self, width: u32, height: u32) -> Result<(), vk::Result>
    {
        unsafe { self.device.device_wait_idle()? };

        // Destroy the old swapchain before creating a new one. On Windows the
        // WSI returns ERROR_NATIVE_WINDOW_IN_USE_KHR if a new swapchain is
        // created on a surface that still owns a live swapchain.
        unsafe
        {
            self.image_views
                .iter()
                .for_each(|view| self.device.destroy_image_view(*view, None));
            self.image_views.clear();
            self.swapchain_loader.destroy_swapchain(self.swapchain_khr, None);
            self.swapchain_khr = vk::SwapchainKHR::null();
        }

        let new_swapchain = Self::create(
            &self.instance,
            &self.device,
            self.graphics_family_index,
            self.present_family_index,
            self.physical_device,
            &self.surface_fn,
            self.surface_khr,
            width,
            height,
        )?;

        *self = new_swapchain;

        Ok(())
    }

    fn create(
        instance: &ash::Instance,
        device: &ash::Device,
        graphics_family_index: u32,
        present_family_index: u32,
        physical_device: vk::PhysicalDevice,
        surface_fn: &ash::khr::surface::Instance,
        surface_khr: vk::SurfaceKHR,
        width: u32,
        height: u32,
    ) -> Result<Self, vk::Result>
    {
        let details = SwapChainSupportDetails::query(surface_fn, surface_khr, physical_device);
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

        let concurrent = graphics_family_index != present_family_index;
        let families_indices = [graphics_family_index, present_family_index];

        let create_info = vk::SwapchainCreateInfoKHR
        {
            surface: surface_khr,
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

        let swapchain_loader = swapchain::Device::new(instance, device);
        let swapchain_khr = unsafe { swapchain_loader.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain_khr).unwrap() };
        let image_views = Self::create_image_views(&images, format.format, device);

        Ok(Self
        {
            swapchain_loader,
            swapchain_khr,
            images,
            image_format: format.format,
            extent,
            image_views,
            device: device.clone(),
            instance: instance.clone(),
            physical_device,
            graphics_family_index,
            present_family_index,
            surface_fn: surface_fn.clone(),
            surface_khr,
        })
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

    pub fn raw_swapchain_khr(&self) -> vk::SwapchainKHR
    {
        self.swapchain_khr
    }

    pub fn loader(&self) -> &swapchain::Device
    {
        &self.swapchain_loader
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

    fn create_image_views(images: &[vk::Image], format: vk::Format, device: &ash::Device) -> Vec<vk::ImageView>
    {
        images
            .iter()
            .map(|image| {
                let create_info = vk::ImageViewCreateInfo
                {
                    image: *image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    components: vk::ComponentMapping
                    {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    },
                    subresource_range: vk::ImageSubresourceRange
                    {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };

                unsafe { device.create_image_view(&create_info, None).unwrap() }
            })
            .collect::<Vec<_>>()
    }
}
impl Drop for SwapChain
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.image_views
                .iter()
                .for_each(|v| self.device.destroy_image_view(*v, None));
            self.swapchain_loader.destroy_swapchain(self.swapchain_khr, None);
        }
    }
}
