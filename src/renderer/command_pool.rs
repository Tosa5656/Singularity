use ash::vk;

use crate::renderer::devices::Device;

pub struct CommandPool
{
    pool: vk::CommandPool,
    device: ash::Device,
}
impl CommandPool
{
    pub fn new(device: &Device) -> Result<Self, vk::Result>
    {
        let command_pool_info = vk::CommandPoolCreateInfo
        {
            queue_family_index: device.graphics_family_index,
            ..Default::default()
        };

        let pool = unsafe { device.device.create_command_pool(&command_pool_info, None).unwrap() };

        Ok(Self { pool, device: device.device.clone() })
    }

    pub fn pool(&self) -> vk::CommandPool
    {
        self.pool
    }
}
impl Drop for CommandPool
{
    fn drop(&mut self)
    {
        unsafe { self.device.destroy_command_pool(self.pool, None) };
    }
}
