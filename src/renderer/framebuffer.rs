use ash::vk;

use crate::renderer::devices::Device;
use crate::renderer::pipelines::GraphicsPipeline;
use crate::renderer::swapchain::SwapChain;

pub struct Framebuffer
{
    pub framebuffers: Vec<vk::Framebuffer>,
    device: ash::Device,
}
impl Framebuffer
{
    pub fn new(device: &Device, swapchain: &SwapChain, graphics_pipeline: &GraphicsPipeline) -> Self
    {
        let framebuffers = swapchain.image_views
            .iter()
            .map(|view| {
                let attachments = [*view];
                let framebuffer_info = vk::FramebufferCreateInfo
                {
                    render_pass: graphics_pipeline.render_pass,
                    attachment_count: 1,
                    p_attachments: attachments.as_ptr(),
                    width: swapchain.extent().width,
                    height: swapchain.extent().height,
                    layers: 1,
                    ..Default::default()
                };
                unsafe { device.device.create_framebuffer(&framebuffer_info, None).unwrap() }
            })
            .collect::<Vec<_>>();

        Self { framebuffers, device: device.device.clone() }
    }
}
impl Drop for Framebuffer
{
    fn drop(&mut self)
    {
        unsafe { self.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None)); };
    }
}
