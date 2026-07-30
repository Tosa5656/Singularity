use ash::vk;

use crate::renderer::devices::Device;

pub struct SyncObjects
{
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    device: ash::Device,
}
impl SyncObjects
{
    pub fn new(device: &Device) -> Self
    {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let image_available_semaphore = unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() };
        let render_finished_semaphore = unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() };

        Self { image_available_semaphore, render_finished_semaphore, device: device.device.clone() }
    }
}
impl Drop for SyncObjects
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.destroy_semaphore(self.render_finished_semaphore, None);
            self.device.destroy_semaphore(self.image_available_semaphore, None);
        }
    }
}
