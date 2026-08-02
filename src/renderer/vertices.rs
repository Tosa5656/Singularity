use ash::vk;

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
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    device: ash::Device,
}
impl VertexBuffer
{
    pub fn new(instance: &Instance, device: &Device, physical_device: &PhysicalDevice, command_pool: &CommandPool, vertices: &[Vertex]) -> Result<Self, vk::Result>
    {
        let memory_properties = unsafe
        {
            instance.instance
                .get_physical_device_memory_properties(physical_device.physical_device)
        };

        let size = (vertices.len() * VERTEX_SIZE) as vk::DeviceSize;

        let (staging_buffer, staging_memory, staging_mem_size) = Self::create_buffer(
            device,
            memory_properties,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        Self::upload_to_buffer(device, staging_memory, staging_mem_size, vertices)?;

        let (buffer, memory, _) = Self::create_buffer(
            device,
            memory_properties,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(device, command_pool, device.graphics_queue, staging_buffer, buffer, size)?;

        unsafe
        {
            device.device.destroy_buffer(staging_buffer, None);
            device.device.free_memory(staging_memory, None);
        };

        Ok(Self { vertex_buffer: buffer, vertex_buffer_memory: memory, device: device.device.clone() })
    }

    fn upload_to_buffer(
        device: &Device,
        memory: vk::DeviceMemory,
        mem_size: vk::DeviceSize,
        vertices: &[Vertex],
    ) -> Result<(), vk::Result>
    {
        unsafe
        {
            let data_ptr = device.device
                .map_memory(memory, 0, mem_size, vk::MemoryMapFlags::empty())?;
            let mut align =
                ash::util::Align::new(data_ptr, std::mem::align_of::<Vertex>() as _, mem_size);
            align.copy_from_slice(vertices);
            device.device.unmap_memory(memory);
        };
        Ok(())
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

    fn create_buffer(
        device: &Device,
        device_mem_properties: vk::PhysicalDeviceMemoryProperties,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory, vk::DeviceSize), vk::Result> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let mem_type =
            Self::find_memory_type(mem_requirements, device_mem_properties, mem_properties);

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type);
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None)? };

        unsafe { device.device.bind_buffer_memory(buffer, memory, 0)? };

        Ok((buffer, memory, mem_requirements.size))
    }

    fn copy_buffer(
        device: &Device,
        command_pool: &CommandPool,
        transfer_queue: vk::Queue,
        src: vk::Buffer,
        dst: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<(), vk::Result>
    {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool.pool())
            .command_buffer_count(1);
        let command_buffers = unsafe { device.device.allocate_command_buffers(&alloc_info)? };
        let command_buffer = command_buffers[0];

        unsafe
        {
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.device.begin_command_buffer(command_buffer, &begin_info)?;

            let region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size,
            };
            let regions = [region];
            device.device.cmd_copy_buffer(command_buffer, src, dst, &regions);

            device.device.end_command_buffer(command_buffer)?;

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&command_buffers);
            let submit_infos = [submit_info];
            device.device.queue_submit(transfer_queue, &submit_infos, vk::Fence::null())?;
            device.device.queue_wait_idle(transfer_queue)?;
        }

        command_pool.free(&command_buffers);
        Ok(())
    }
}
impl Drop for VertexBuffer
{
    fn drop(&mut self)
    {
        unsafe
        {
            let _ = self.device.device_wait_idle();
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
        }
    }
}
