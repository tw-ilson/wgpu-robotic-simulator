use bytemuck::{Pod,Zeroable};


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    position: glm::Vec3,
    _padding: u32,
    color: glm::Vec3,
    _padding2: u32,
}
unsafe impl Pod for LightUniform {}
unsafe impl Zeroable for LightUniform {}
