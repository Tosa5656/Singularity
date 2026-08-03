use ash::vk;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::renderer::devices::Device;
use crate::renderer::swapchain::SwapChain;
use crate::renderer::sync::InFlightFrames;
use crate::renderer::command_buffer::CommandBuffer;
use crate::renderer::command_pool::CommandPool;
use crate::renderer::pipelines::*;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::buffers::VertexBuffer;

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
    window_size: Option<[u32; 2]>,
    swapchain: SwapChain,
    graphics_pipeline: GraphicsPipeline,
    framebuffer: Framebuffer,
    in_flight_frames: InFlightFrames,
    command_buffers: Vec<CommandBuffer>,
    command_pool: CommandPool,
    vertex_buffer: VertexBuffer,
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
        in_flight_frames: InFlightFrames,
        command_buffers: Vec<CommandBuffer>,
        command_pool: CommandPool,
        vertex_buffer: VertexBuffer,
        device: Device,
    ) -> Self
    {
        let window_size =
        {
            let size = window.inner_size();
            Some([size.width, size.height])
        };

        Self { event_loop, window, window_size, swapchain, graphics_pipeline, framebuffer, in_flight_frames, command_buffers, command_pool, vertex_buffer, device }
    }

    pub fn run(self)
    {
        let device_raw = self.device.device.clone();
        let graphics_queue = self.device.graphics_queue;
        let present_queue = self.device.present_queue;
        let mut swapchain = self.swapchain;
        let graphics_pipeline = self.graphics_pipeline;
        let mut framebuffer = self.framebuffer;
        let mut in_flight_frames = self.in_flight_frames;
        let mut command_buffers = self.command_buffers;
        let command_pool = self.command_pool;
        let vertex_buffer = self.vertex_buffer;
        let mut window_size = self.window_size;
        let window = self.window;
        let _idle_guard = DeviceIdleGuard(device_raw.clone());

        let swapchain = &mut swapchain;
        let graphics_pipeline = &graphics_pipeline;
        let framebuffer = &mut framebuffer;
        let in_flight_frames = &mut in_flight_frames;
        let command_buffers = &mut command_buffers;
        let command_pool = &command_pool;
        let vertex_buffer = &vertex_buffer;

        self.event_loop.run(move |event, elwt|
        {
            if let Event::WindowEvent { event, window_id } = event
            {
                if window_id == window.id()
                {
                    match event
                    {
                        WindowEvent::CloseRequested =>
                        {
                            elwt.exit();
                            return;
                        }
                        WindowEvent::Resized(size) =>
                        {
                            window_size = Some([size.width, size.height]);
                        }
                        _ => {}
                    }
                }
            }

            let size = window_size.unwrap_or_else(||
            {
                let extent = swapchain.extent();
                [extent.width, extent.height]
            });

            draw_frame(
                &device_raw,
                in_flight_frames,
                swapchain,
                framebuffer,
                command_pool,
                command_buffers,
                graphics_pipeline,
                graphics_queue,
                present_queue,
                vertex_buffer,
                size,
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
    in_flight_frames: &mut InFlightFrames,
    swapchain: &mut SwapChain,
    framebuffer: &mut Framebuffer,
    command_pool: &CommandPool,
    command_buffers: &mut Vec<CommandBuffer>,
    graphics_pipeline: &GraphicsPipeline,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    vertex_buffer: &VertexBuffer,
    window_size: [u32; 2],
)
{
    if window_size[0] == 0 || window_size[1] == 0
    {
        return;
    }

    let frame = in_flight_frames.current();
    let in_flight_fence = frame.in_flight_fence;
    let image_available_semaphore = frame.image_available_semaphore;

    unsafe
    {
        device.wait_for_fences(&[in_flight_fence], true, std::u64::MAX).unwrap();
        device.reset_fences(&[in_flight_fence]).unwrap();
    }

    let image_index = match unsafe
    {
        swapchain
            .loader()
            .acquire_next_image(
                swapchain.raw_swapchain_khr(),
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            )
    }
    {
        Ok((image_index, _)) => image_index as usize,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) =>
        {
            recreate_swapchain(device, in_flight_frames, swapchain, framebuffer, command_pool, command_buffers, graphics_pipeline, vertex_buffer, window_size);
            return;
        }
        Err(error) => panic!("Failed to acquire next image. Cause: {}", error),
    };

    let render_finished_semaphore = in_flight_frames.render_finished_semaphore(image_index);

    let wait_semaphores = [image_available_semaphore];
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let signal_semaphores = [render_finished_semaphore];
    let cmd_buffers = [command_buffers[image_index].buffer];

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

    let swapchains = [swapchain.raw_swapchain_khr()];
    let image_indices = [image_index as u32];

    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);

    match unsafe
    {
        swapchain.loader().queue_present(present_queue, &present_info)
    }
    {
        Ok(is_suboptimal) if is_suboptimal =>
        {
            recreate_swapchain(device, in_flight_frames, swapchain, framebuffer, command_pool, command_buffers, graphics_pipeline, vertex_buffer, window_size);
        }
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) =>
        {
            recreate_swapchain(device, in_flight_frames, swapchain, framebuffer, command_pool, command_buffers, graphics_pipeline, vertex_buffer, window_size);
        }
        Err(error) => panic!("Failed to present queue. Cause: {}", error),
        _ => {}
    }

    in_flight_frames.advance();
}

fn recreate_swapchain(
    device: &ash::Device,
    in_flight_frames: &mut InFlightFrames,
    swapchain: &mut SwapChain,
    framebuffer: &mut Framebuffer,
    command_pool: &CommandPool,
    command_buffers: &mut Vec<CommandBuffer>,
    graphics_pipeline: &GraphicsPipeline,
    vertex_buffer: &VertexBuffer,
    window_size: [u32; 2],
)
{
    if window_size[0] == 0 || window_size[1] == 0
    {
        return;
    }

    swapchain.recreate(window_size[0], window_size[1])
        .expect("Failed to recreate swapchain");

    framebuffer.recreate(swapchain, graphics_pipeline.render_pass);

    let old_buffers: Vec<vk::CommandBuffer> = command_buffers
        .iter()
        .map(|cb| cb.buffer)
        .collect();
    command_pool.free(&old_buffers);

    *command_buffers = framebuffer.framebuffers
        .iter()
        .map(|fb| CommandBuffer::new(
            device,
            command_pool.pool(),
            *fb,
            graphics_pipeline.render_pass,
            swapchain.extent(),
            graphics_pipeline.pipeline,
            vertex_buffer.buffer.get(),
        ))
        .collect();

    in_flight_frames.recreate(swapchain.image_views.len())
        .expect("Failed to recreate sync objects");
}
