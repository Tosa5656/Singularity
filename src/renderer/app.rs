use ash::vk;
use ash::khr::swapchain;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::renderer::devices::Device;
use crate::renderer::swapchain::SwapChain;
use crate::renderer::sync::SyncObjects;
use crate::renderer::command_buffer::CommandBuffer;
use crate::renderer::command_pool::CommandPool;
use crate::renderer::pipelines::GraphicsPipeline;
use crate::renderer::framebuffer::Framebuffer;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const REQUIRED_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct AppInfo
{
    pub app_name: std::ffi::CString,
    pub app_version: u32,
    pub api_version: u32,
}

pub struct App
{
    pub event_loop: EventLoop<()>,
    pub window: Window,
    swapchain: SwapChain,
    graphics_pipeline: GraphicsPipeline,
    framebuffer: Framebuffer,
    sync_objects: SyncObjects,
    command_buffers: Vec<CommandBuffer>,
    command_pool: CommandPool,
    device: Device,
}

impl App
{
    pub fn new(
        event_loop: EventLoop<()>,
        window: Window,
        swapchain: SwapChain,
        graphics_pipeline: GraphicsPipeline,
        framebuffer: Framebuffer,
        sync_objects: SyncObjects,
        command_buffers: Vec<CommandBuffer>,
        command_pool: CommandPool,
        device: Device,
    ) -> Self
    {
        Self { event_loop, window, swapchain, graphics_pipeline, framebuffer, sync_objects, command_buffers, command_pool, device }
    }

    pub fn run(self)
    {
        let device_raw = self.device.device.clone();
        let device_for_cleanup = device_raw.clone();
        let graphics_queue = self.device.graphics_queue;
        let present_queue = self.device.present_queue;
        let swapchain_loader = self.swapchain.loader().clone();
        let swapchain_khr = self.swapchain.raw_swapchain_khr();
        let image_available_semaphore = self.sync_objects.image_available_semaphore;
        let render_finished_semaphore = self.sync_objects.render_finished_semaphore;
        let cmd_buffers: Vec<vk::CommandBuffer> = self.command_buffers.into_iter().map(|cb| cb.buffer).collect();
        let window = self.window;

        self.event_loop.run(move |event, elwt|
        {
            if let Event::WindowEvent { event, window_id } = event
            {
                if window_id == window.id()
                {
                    if matches!(event, WindowEvent::CloseRequested)
                    {
                        elwt.exit();
                        return;
                    }
                }
            }
            draw_frame(
                &device_raw,
                &swapchain_loader,
                swapchain_khr,
                graphics_queue,
                present_queue,
                image_available_semaphore,
                render_finished_semaphore,
                &cmd_buffers,
            );
        }).expect("Event loop error");

        unsafe { device_for_cleanup.device_wait_idle().unwrap(); }
    }
}

fn draw_frame(
    device: &ash::Device,
    swapchain_loader: &swapchain::Device,
    swapchain_khr: vk::SwapchainKHR,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    command_buffers: &[vk::CommandBuffer],
)
{
    unsafe { device.device_wait_idle().unwrap(); }

    let image_index = unsafe
    {
        swapchain_loader
            .acquire_next_image(
                swapchain_khr,
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            )
            .unwrap()
            .0 as usize
    };

    let wait_semaphores = [image_available_semaphore];
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let signal_semaphores = [render_finished_semaphore];
    let cmd_buffers = [command_buffers[image_index]];

    let submit_info = vk::SubmitInfo::default()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&wait_stages)
        .command_buffers(&cmd_buffers)
        .signal_semaphores(&signal_semaphores);

    unsafe
    {
        device
            .queue_submit(graphics_queue, &[submit_info], vk::Fence::null())
            .unwrap();
    }

    let swapchains = [swapchain_khr];
    let image_indices = [image_index as u32];

    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);

    unsafe
    {
        swapchain_loader
            .queue_present(present_queue, &present_info)
            .unwrap();
    }
}
