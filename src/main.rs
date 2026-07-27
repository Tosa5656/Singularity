use std::ffi::CString;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use Singularity::renderer::app::AppInfo;
use Singularity::renderer::instance::Instance;
use Singularity::renderer::devices::{PhysicalDevice, Device};

fn main() -> Result<(), Box<dyn std::error::Error>>
{
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

    let physical_device = PhysicalDevice::new(&instance)
        .expect("Failed to pick physical device");

    let device = Device::new(&instance, &physical_device)
        .expect("Failed to create logical device");
    
    event_loop.run(move |event, elwt|
    {
        if let Event::WindowEvent { event, window_id } = event
        {
            if window_id == window.id()
            {
                match event
                {
                    WindowEvent::CloseRequested => elwt.exit(),
                    _ => {}
                }
            }
        }
    })?;

    Ok(())
}
