use ash::vk;
use std::path::Path;

use crate::renderer::devices::Device;

pub struct Shader<'a>
{
    pub shader_module: vk::ShaderModule,
    pub device: &'a Device,
}
impl<'a> Shader<'a>
{
    pub fn new(device: &'a Device, path: &Path) -> Result<Self, vk::Result>
    {
        let shader_source = Self::read_file(path);

        let create_info = vk::ShaderModuleCreateInfo
        {
            code_size: shader_source.len() * 4,
            p_code: shader_source.as_ptr(),
            ..Default::default()
        };
        let shader_module = unsafe { device.device.create_shader_module(&create_info, None).unwrap() };

        Ok(Self { shader_module, device })
    }

    fn read_file(path: &Path) -> Vec<u32>
    {
        let mut file = std::fs::File::open(path).unwrap();
        ash::util::read_spv(&mut file).unwrap()
    }
}
impl Drop for Shader<'_>
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.device.destroy_shader_module(self.shader_module, None);
        }
    }
}
