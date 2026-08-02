use ash::vk;

use crate::renderer::buffer::Buffer;
use crate::renderer::{command_pool::CommandPool, devices::{Device, PhysicalDevice}, instance::Instance};

const VERTEX_SIZE: usize = 24;

#[derive(Copy, Clone)]
pub struct Vertex
{
    pub position: [f32; 3],
    pub color: [f32; 3]
}
impl Vertex
{
    pub fn get_binding_description() -> vk::VertexInputBindingDescription
    {
        vk::VertexInputBindingDescription
        {
            binding: 0,
            stride: VERTEX_SIZE as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2]
    {
        let position_desc = vk::VertexInputAttributeDescription
        {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        };
        let color_desc = vk::VertexInputAttributeDescription
        {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
        };
        [position_desc, color_desc]
    }
}

pub struct VertexBuffer
{
    pub buffer: Buffer,
}
impl VertexBuffer
{
    pub fn new(instance: &Instance, device: &Device, physical_device: &PhysicalDevice, command_pool: &CommandPool, vertices: &[Vertex]) -> Result<Self, vk::Result>
    {
        let size = (vertices.len() * VERTEX_SIZE) as vk::DeviceSize;

        let staging_buffer = Buffer::new(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        staging_buffer.upload_data(vertices)?;

        let buffer = Buffer::new(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        buffer.copy_from(command_pool, device.graphics_queue, &staging_buffer)?;

        Ok(Self { buffer })
    }
}
