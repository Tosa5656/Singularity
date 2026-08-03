use std::mem::size_of;
use ash::vk;

use crate::renderer::command_pool::CommandPool;
use crate::renderer::devices::{Device, PhysicalDevice};
use crate::renderer::instance::Instance;
use crate::renderer::vertices::Vertex;

pub struct Buffer
{
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    device: ash::Device,
}
impl Buffer
{
    pub fn new(
        instance: &Instance,
        device: &Device,
        physical_device: &PhysicalDevice,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Result<Self, vk::Result>
    {
        let memory_properties = unsafe
        {
            instance.instance
                .get_physical_device_memory_properties(physical_device.physical_device)
        };

        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let mem_type =
            Self::find_memory_type(mem_requirements, memory_properties, mem_properties);

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type);
        let memory = unsafe { device.device.allocate_memory(&alloc_info, None)? };

        unsafe { device.device.bind_buffer_memory(buffer, memory, 0)? };

        Ok(Self { buffer, memory, size, device: device.device.clone() })
    }

    pub fn get(&self) -> vk::Buffer
    {
        self.buffer
    }

    pub fn size(&self) -> vk::DeviceSize
    {
        self.size
    }

    pub fn upload_data<T: Copy>(&self, data: &[T]) -> Result<(), vk::Result>
    {
        unsafe
        {
            let data_ptr = self.device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())?;
            let mut align =
                ash::util::Align::new(data_ptr, std::mem::align_of::<T>() as _, self.size);
            align.copy_from_slice(data);
            self.device.unmap_memory(self.memory);
        };
        Ok(())
    }

    pub fn copy_from(
        &self,
        command_pool: &CommandPool,
        transfer_queue: vk::Queue,
        src: &Buffer,
    ) -> Result<(), vk::Result>
    {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool.pool())
            .command_buffer_count(1);
        let command_buffers = unsafe { self.device.allocate_command_buffers(&alloc_info)? };
        let command_buffer = command_buffers[0];

        unsafe
        {
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            self.device.begin_command_buffer(command_buffer, &begin_info)?;

            let region = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: self.size,
            };
            let regions = [region];
            self.device.cmd_copy_buffer(command_buffer, src.buffer, self.buffer, &regions);

            self.device.end_command_buffer(command_buffer)?;

            let submit_info = vk::SubmitInfo::default()
                .command_buffers(&command_buffers);
            let submit_infos = [submit_info];
            self.device.queue_submit(transfer_queue, &submit_infos, vk::Fence::null())?;
            self.device.queue_wait_idle(transfer_queue)?;
        }

        command_pool.free(&command_buffers);
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
}
impl Drop for Buffer
{
    fn drop(&mut self)
    {
        unsafe
        {
            let _ = self.device.device_wait_idle();
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
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
        let size = (vertices.len() * size_of::<Vertex>()) as vk::DeviceSize;

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

pub struct IndexBuffer
{
    pub buffer: Buffer,
    index_count: u32,
}
impl IndexBuffer
{
    pub fn new(instance: &Instance, device: &Device, physical_device: &PhysicalDevice, command_pool: &CommandPool, indices: &[u32]) -> Result<Self, vk::Result>
    {
        let size = (indices.len() * size_of::<u32>()) as vk::DeviceSize;

        let staging_buffer = Buffer::new(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        staging_buffer.upload_data(indices)?;

        let buffer = Buffer::new(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        buffer.copy_from(command_pool, device.graphics_queue, &staging_buffer)?;

        Ok(Self { buffer, index_count: indices.len() as u32 })
    }

    pub fn index_count(&self) -> u32
    {
        self.index_count
    }
}