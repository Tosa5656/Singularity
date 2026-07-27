use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Surface
{
    pub surface_fn: ash::khr::surface::Instance,
    pub surface_khr: vk::SurfaceKHR
}
impl Surface
{
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, window: &(impl HasDisplayHandle + HasWindowHandle)) -> Result<Self, vk::Result>
    {
        let raw_display = window.display_handle().unwrap().as_raw();
        let raw_window = window.window_handle().unwrap().as_raw();
        let surface_khr = unsafe {
            ash_window::create_surface(entry, instance, raw_display, raw_window, None)?
        };
        let surface_fn = ash::khr::surface::Instance::new(entry, instance);

        Ok(Self { surface_fn, surface_khr })
    }

    pub fn get_required_extensions(window: &impl HasDisplayHandle) -> Vec<*const std::os::raw::c_char>
    {
        let raw_display = window.display_handle().unwrap().as_raw();
        let raw_extensions = ash_window::enumerate_required_extensions(raw_display).unwrap();
        raw_extensions.iter().copied().collect()
    }

    pub fn get_physical_device_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool
    {
        unsafe {
            self.surface_fn
                .get_physical_device_surface_support(physical_device, queue_family_index, self.surface_khr)
                .unwrap()
        }
    }

    pub fn get(&self) -> vk::SurfaceKHR
    {
        self.surface_khr
    }
}
impl Drop for Surface
{
    fn drop(&mut self)
    {
        unsafe { self.surface_fn.destroy_surface(self.surface_khr, None); }
    }
}
