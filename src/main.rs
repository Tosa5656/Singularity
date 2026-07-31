use std::ffi::CString;
use std::path::Path;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::env;

use Singularity::renderer::{app::{App, AppInfo}, sync::InFlightFrames};
use Singularity::renderer::instance::Instance;
use Singularity::renderer::surface::Surface;
use Singularity::renderer::devices::{PhysicalDevice, Device};
use Singularity::renderer::swapchain::SwapChain;
use Singularity::renderer::shaders::Shader;
use Singularity::renderer::pipelines::GraphicsPipeline;
use Singularity::renderer::framebuffer::Framebuffer;
use Singularity::renderer::command_pool::CommandPool;
use Singularity::renderer::command_buffer::CommandBuffer;

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    if let Ok(mut exe_path) = env::current_exe()
    {
        exe_path.pop();
        env::set_current_dir(&exe_path).expect("Failed to set workdir");
    }

    env_logger::init();

    let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan") };

    // Window
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window_attrs = Window::default_attributes()
        .with_title("Singularity")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
    let window = event_loop.create_window(window_attrs)?;

    let app_info = AppInfo {
        app_name: CString::new("Singularity").unwrap(),
        app_version: ash::vk::make_api_version(0, 0, 1, 0),
        api_version: ash::vk::make_api_version(0, 1, 4, 0),
    };

    let instance = Instance::new(&entry, app_info, &window)
        .expect("Failed to create Vulkan instance");

    let surface = Surface::new(&entry, &instance.instance, &window)
        .expect("Failed to create surface");

    let physical_device = PhysicalDevice::new(&instance, &surface)
        .expect("Failed to pick physical device");

    let device = Device::new(&instance, &physical_device, &surface)
        .expect("Failed to create logical device");

    let window_size = window.inner_size();
    let swapchain = SwapChain::new(
        &instance,
        &physical_device,
        &device,
        &surface,
        window_size.width,
        window_size.height,
    ).expect("Failed to create swapchain");

    let vertex_shader = Shader::new(&device, Path::new("shaders/base.vert.spv"))
        .expect("Failed to create vertex shader");
    let fragment_shader = Shader::new(&device, Path::new("shaders/base.frag.spv"))
        .expect("Failed to create fragment shader");

    let graphics_pipeline = GraphicsPipeline::new(&device, &swapchain, &vertex_shader, &fragment_shader)
        .expect("Failed to create graphics pipeline");

    let framebuffer = Framebuffer::new(&device, &swapchain, &graphics_pipeline);

    let command_pool = CommandPool::new(&device)
        .expect("Failed to create command pool");

    let command_buffers: Vec<CommandBuffer> = framebuffer.framebuffers
        .iter()
        .map(|fb| CommandBuffer::new(
            &device,
            command_pool.pool(),
            *fb,
            graphics_pipeline.render_pass,
            swapchain.extent(),
            graphics_pipeline.pipeline,
        ))
        .collect();

    let in_flight_frames = InFlightFrames::new(&device, swapchain.image_views.len())
        .expect("Failed to create sync objects");

    drop(vertex_shader);
    drop(fragment_shader);

    let app = App::new(event_loop, window, swapchain, graphics_pipeline, framebuffer, in_flight_frames, command_buffers, command_pool, device);
    app.run();

    Ok(())
}
