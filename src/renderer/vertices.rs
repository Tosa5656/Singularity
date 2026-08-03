use ash::vk;

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