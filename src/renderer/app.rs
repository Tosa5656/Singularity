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
        let graphics_queue = self.device.graphics_queue;
        let present_queue = self.device.present_queue;
        let swapchain_loader = self.swapchain.loader().clone();
        let swapchain_khr = self.swapchain.raw_swapchain_khr();
        let mut sync_objects = self.sync_objects;
        let cmd_buffers: Vec<vk::CommandBuffer> = self.command_buffers.into_iter().map(|cb| cb.buffer).collect();
        let window = self.window;
        let _idle_guard = DeviceIdleGuard(device_raw.clone());

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
                &mut sync_objects,
                &swapchain_loader,
                swapchain_khr,
                graphics_queue,
                present_queue,
                &cmd_buffers,
            );
        })
        .expect("Event loop error");
    }
}

struct DeviceIdleGuard(ash::Device);

impl Drop for DeviceIdleGuard
{
    fn drop(&mut self)
    {
        unsafe { let _ = self.0.device_wait_idle(); }
    }
}

fn draw_frame(
    device: &ash::Device,
    sync_objects: &mut SyncObjects,
    swapchain_loader: &swapchain::Device,
    swapchain_khr: vk::SwapchainKHR,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_buffers: &[vk::CommandBuffer],
)
{
    let current_frame = sync_objects.current_frame;
    let in_flight_fence = sync_objects.in_flight_fences[current_frame];
    let image_available_semaphore = sync_objects.image_available_semaphores[current_frame];

    unsafe
    {
        device.wait_for_fences(&[in_flight_fence], true, std::u64::MAX).unwrap();
        device.reset_fences(&[in_flight_fence]).unwrap();
    }

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

    let render_finished_semaphore = sync_objects.render_finished_semaphores[image_index];

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
            .queue_submit(graphics_queue, &[submit_info], in_flight_fence)
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

    sync_objects.current_frame = (current_frame + 1) % SyncObjects::MAX_FRAMES_IN_FLIGHT;
}
