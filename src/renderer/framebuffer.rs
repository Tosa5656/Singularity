use ash::vk;

use crate::renderer::pipelines::GraphicsPipeline;
use crate::renderer::swapchain::SwapChain;

pub struct Framebuffer
{
    pub framebuffers: Vec<vk::Framebuffer>,
    device: ash::Device,
}
impl Framebuffer
{
    pub fn new(device: &ash::Device, swapchain: &SwapChain, graphics_pipeline: &GraphicsPipeline) -> Self
    {
        Self
        {
            framebuffers: Self::create_framebuffers(device, swapchain, graphics_pipeline.render_pass),
            device: device.clone(),
        }
    }

    pub fn recreate(&mut self, swapchain: &SwapChain, render_pass: vk::RenderPass)
    {
        unsafe
        {
            self.framebuffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));
        }

        self.framebuffers = Self::create_framebuffers(&self.device, swapchain, render_pass);
    }

    fn create_framebuffers(device: &ash::Device, swapchain: &SwapChain, render_pass: vk::RenderPass) -> Vec<vk::Framebuffer>
    {
        swapchain.image_views
            .iter()
            .map(|view| {
                let attachments = [*view];
                let framebuffer_info = vk::FramebufferCreateInfo
                {
                    render_pass,
                    attachment_count: 1,
                    p_attachments: attachments.as_ptr(),
                    width: swapchain.extent().width,
                    height: swapchain.extent().height,
                    layers: 1,
                    ..Default::default()
                };
                unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() }
            })
            .collect::<Vec<_>>()
    }
}
impl Drop for Framebuffer
{
    fn drop(&mut self)
    {
        unsafe { self.framebuffers.iter().for_each(|f| self.device.destroy_framebuffer(*f, None)); };
    }
}
