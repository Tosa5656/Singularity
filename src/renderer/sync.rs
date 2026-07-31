use ash::vk;

pub struct SyncObjects
{
    pub image_available_semaphore: vk::Semaphore,
    pub in_flight_fence: vk::Fence,
    device: ash::Device,
}
impl SyncObjects
{
    pub fn new(device: &ash::Device) -> Result<Self, vk::Result>
    {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo
        {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_info, None)? };
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None)? };

        Ok(Self { image_available_semaphore, in_flight_fence, device: device.clone() })
    }
}
impl Drop for SyncObjects
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.destroy_fence(self.in_flight_fence, None);
            self.device.destroy_semaphore(self.image_available_semaphore, None);
        }
    }
}

pub struct InFlightFrames
{
    frames: Vec<SyncObjects>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    current_frame: usize,
    device: ash::Device,
}
impl InFlightFrames
{
    pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(device: &ash::Device, swapchain_image_count: usize) -> Result<Self, vk::Result>
    {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let frames = (0..Self::MAX_FRAMES_IN_FLIGHT)
            .map(|_| SyncObjects::new(device))
            .collect::<Result<Vec<_>, _>>()?;

        let render_finished_semaphores = (0..swapchain_image_count)
            .map(|_| unsafe { device.create_semaphore(&semaphore_info, None) })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { frames, render_finished_semaphores, current_frame: 0, device: device.clone() })
    }

    pub fn recreate(&mut self, swapchain_image_count: usize) -> Result<(), vk::Result>
    {
        *self = Self::new(&self.device, swapchain_image_count)?;
        Ok(())
    }

    pub fn current(&self) -> &SyncObjects
    {
        &self.frames[self.current_frame]
    }

    pub fn render_finished_semaphore(&self, image_index: usize) -> vk::Semaphore
    {
        self.render_finished_semaphores[image_index]
    }

    pub fn advance(&mut self)
    {
        self.current_frame = (self.current_frame + 1) % Self::MAX_FRAMES_IN_FLIGHT;
    }
}
impl Drop for InFlightFrames
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.render_finished_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
        }
    }
}
