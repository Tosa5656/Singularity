use ash::vk;

use crate::renderer::devices::Device;

pub struct SyncObjects
{
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
    device: ash::Device,
}
impl SyncObjects
{
    pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(device: &Device, swapchain_image_count: usize) -> Self
    {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let image_available_semaphores = (0..Self::MAX_FRAMES_IN_FLIGHT)
            .map(|_| unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() })
            .collect::<Vec<_>>();

        let render_finished_semaphores = (0..swapchain_image_count)
            .map(|_| unsafe { device.device.create_semaphore(&semaphore_info, None).unwrap() })
            .collect::<Vec<_>>();

        let fence_info = vk::FenceCreateInfo
        {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };
        let in_flight_fences = (0..Self::MAX_FRAMES_IN_FLIGHT)
            .map(|_| unsafe { device.device.create_fence(&fence_info, None).unwrap() })
            .collect::<Vec<_>>();

        Self { image_available_semaphores, render_finished_semaphores, in_flight_fences, current_frame: 0, device: device.device.clone() }
    }
}
impl Drop for SyncObjects
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.in_flight_fences
                .iter()
                .for_each(|f| self.device.destroy_fence(*f, None));
            self.render_finished_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.image_available_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
        }
    }
}
