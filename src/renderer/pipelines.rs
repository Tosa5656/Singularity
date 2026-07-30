use ash::vk;
use std::ffi::CString;

use crate::renderer::devices::Device;
use crate::renderer::swapchain::SwapChain;
use crate::renderer::shaders::Shader;

pub struct GraphicsPipeline<'a>
{
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub device: &'a Device,
}
impl<'a> GraphicsPipeline<'a>
{
    pub fn new(
        device: &'a Device,
        swapchain: &SwapChain,
        vertex_shader: &Shader,
        fragment_shader: &Shader,
    ) -> Result<Self, vk::Result>
    {
        let attachment_desc = vk::AttachmentDescription
        {
            format: swapchain.image_format(),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };
        let attachment_descs = [attachment_desc];

        let attachment_ref = vk::AttachmentReference
        {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let attachment_refs = [attachment_ref];

        let subpass_desc = vk::SubpassDescription
        {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: attachment_refs.as_ptr(),
            ..Default::default()
        };
        let subpass_descs = [subpass_desc];

        let render_pass_info = vk::RenderPassCreateInfo
        {
            attachment_count: 1,
            p_attachments: attachment_descs.as_ptr(),
            subpass_count: 1,
            p_subpasses: subpass_descs.as_ptr(),
            ..Default::default()
        };

        let render_pass = unsafe { device.device.create_render_pass(&render_pass_info, None).unwrap() };

        let entry_point_name = CString::new("main").unwrap();
        let vertex_shader_state_info = vk::PipelineShaderStageCreateInfo
        {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vertex_shader.shader_module,
            p_name: entry_point_name.as_ptr(),
            ..Default::default()
        };
        let fragment_shader_state_info = vk::PipelineShaderStageCreateInfo
        {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: fragment_shader.shader_module,
            p_name: entry_point_name.as_ptr(),
            ..Default::default()
        };
        let shader_states_infos = [vertex_shader_state_info, fragment_shader_state_info];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo
        {
            ..Default::default()
        };

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo
        {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport = vk::Viewport
        {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent().width as _,
            height: swapchain.extent().height as _,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let viewports = [viewport];
        let scissor = vk::Rect2D
        {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent(),
        };
        let scissors = [scissor];
        let viewport_info = vk::PipelineViewportStateCreateInfo
        {
            viewport_count: 1,
            p_viewports: viewports.as_ptr(),
            scissor_count: 1,
            p_scissors: scissors.as_ptr(),
            ..Default::default()
        };

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo
        {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo
        {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0,
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState
        {
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };
        let color_blend_attachments = [color_blend_attachment];

        let color_blending_info = vk::PipelineColorBlendStateCreateInfo
        {
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: color_blend_attachments.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };

        let pipeline_layout =
        {
            let pipeline_layout_info = vk::PipelineLayoutCreateInfo
            {
                ..Default::default()
            };

            unsafe { device.device.create_pipeline_layout(&pipeline_layout_info, None).unwrap() }
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo
        {
            stage_count: 2,
            p_stages: shader_states_infos.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly_info,
            p_viewport_state: &viewport_info,
            p_rasterization_state: &rasterizer_info,
            p_multisample_state: &multisampling_info,
            p_color_blend_state: &color_blending_info,
            layout: pipeline_layout,
            render_pass,
            subpass: 0,
            ..Default::default()
        };
        let pipeline_infos = [pipeline_info];

        let pipeline = unsafe
        {
            device.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .unwrap()[0]
        };

        Ok(Self { pipeline, pipeline_layout, render_pass, device })
    }
}
impl Drop for GraphicsPipeline<'_>
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.device.destroy_pipeline(self.pipeline, None);
            self.device.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
