use ash::vk;

use crate::renderer::devices::Device;

pub struct CommandPool
{
    pool: vk::CommandPool,
    device: ash::Device,
}
impl CommandPool
{
    pub fn new(device: &Device, create_flags: vk::CommandPoolCreateFlags) -> Result<Self, vk::Result>
    {
        let command_pool_info = vk::CommandPoolCreateInfo
        {
            queue_family_index: device.graphics_family_index,
            flags: create_flags,
            ..Default::default()
        };

        let pool = unsafe { device.device.create_command_pool(&command_pool_info, None).unwrap() };

        Ok(Self { pool, device: device.device.clone() })
    }

    pub fn pool(&self) -> vk::CommandPool
    {
        self.pool
    }

    pub fn free(&self, buffers: &[vk::CommandBuffer])
    {
        unsafe { self.device.free_command_buffers(self.pool, buffers) };
    }
}
impl Drop for CommandPool
{
    fn drop(&mut self)
    {
        unsafe { self.device.destroy_command_pool(self.pool, None) };
    }
}
