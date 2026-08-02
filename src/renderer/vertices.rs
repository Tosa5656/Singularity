use ash::vk;

use crate::renderer::{devices::{Device, PhysicalDevice}, instance::Instance};

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
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    device: ash::Device,
}
impl VertexBuffer
{
    pub fn new(instance: &Instance, device: &Device, physical_device: &PhysicalDevice, vertices: &[Vertex]) -> Result<Self, vk::Result>
    {
        let memory_properties = unsafe
        {
            instance
                .instance
                .get_physical_device_memory_properties(physical_device.physical_device)
        };

        let buffer_size = (vertices.len() * VERTEX_SIZE) as vk::DeviceSize;

        let buffer_info = vk::BufferCreateInfo
        {
            size: buffer_size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe { device.device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let mem_type = Self::find_memory_type(
            mem_requirements,
            memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        let alloc_info = vk::MemoryAllocateInfo
        {
            allocation_size: mem_requirements.size,
            memory_type_index: mem_type,
            ..Default::default()
        };
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None)? };

        unsafe
        {
            device.device.bind_buffer_memory(buffer, memory, 0)?;

            let data_ptr = device
                .device
                .map_memory(memory, 0, mem_requirements.size, vk::MemoryMapFlags::empty())?;
            let mut align = ash::util::Align::new(
                data_ptr,
                std::mem::align_of::<Vertex>() as _,
                mem_requirements.size,
            );
            align.copy_from_slice(&vertices);
            device.device.unmap_memory(memory);
        };

        Ok(Self { vertex_buffer: buffer, vertex_buffer_memory: memory, device: device.device.clone() })
    }

    fn find_memory_type(
        requirements: vk::MemoryRequirements,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
        required_properties: vk::MemoryPropertyFlags,
    ) -> u32
    {
        for i in 0..mem_properties.memory_type_count
        {
            if requirements.memory_type_bits & (1 << i) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(required_properties)
            {
                return i;
            }
        }
        panic!("Failed to find suitable memory type.")
    }
}
impl Drop for VertexBuffer
{
    fn drop(&mut self)
    {
        unsafe
        {
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
        }
    }
}
