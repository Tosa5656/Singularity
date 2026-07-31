use ash::vk;

pub struct CommandBuffer
{
    pub buffer: vk::CommandBuffer,
}

impl CommandBuffer
{
    pub fn new(
        device: &ash::Device,
        pool: vk::CommandPool,
        framebuffer: vk::Framebuffer,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        graphics_pipeline: vk::Pipeline,
        vertex_buffer: vk::Buffer,
    ) -> Self
    {
        let allocate_info = vk::CommandBufferAllocateInfo
        {
            command_pool: pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };

        let buffer = unsafe { device.allocate_command_buffers(&allocate_info).unwrap()[0] };

        {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo
            {
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                ..Default::default()
            };
            unsafe { device.begin_command_buffer(buffer, &command_buffer_begin_info).unwrap() };
        }

        {
            let clear_values = [vk::ClearValue
            {
                color: vk::ClearColorValue
                {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];
            let render_pass_begin_info = vk::RenderPassBeginInfo
            {
                render_pass,
                framebuffer,
                render_area: vk::Rect2D
                {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent,
                },
                clear_value_count: 1,
                p_clear_values: clear_values.as_ptr(),
                ..Default::default()
            };

            unsafe
            {
                device.cmd_begin_render_pass(
                    buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                )
            };
        }

        unsafe { device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline) };

        let vertex_buffers = [vertex_buffer];
        let offsets = [0 as vk::DeviceSize];
        unsafe { device.cmd_bind_vertex_buffers(buffer, 0, &vertex_buffers, &offsets) };

        let viewport = vk::Viewport
        {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        unsafe { device.cmd_set_viewport(buffer, 0, &[viewport]) };

        let scissor = vk::Rect2D
        {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };
        unsafe { device.cmd_set_scissor(buffer, 0, &[scissor]) };

        unsafe { device.cmd_draw(buffer, 3, 1, 0, 0) };

        unsafe { device.cmd_end_render_pass(buffer) };

        unsafe { device.end_command_buffer(buffer).unwrap() };

        Self { buffer }
    }
}
