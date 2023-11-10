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

impl LightUniform {
    pub fn new() -> Self {
        Self { position: glm::vec3(0., 0., 0.0), _padding: 0, color: glm::vec3(1.,1.,1.), _padding2: 0 }
    }
}
