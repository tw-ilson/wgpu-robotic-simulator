use bytemuck::{Pod,Zeroable};

#[derive(Debug)]
pub struct Light {
    ambient_intensity: f32,
    pub uniform: LightUniform,
    // buffer: wgpu::Buffer,
}
impl Light {
    pub fn new(intensity: Option<f32>) -> Self {
        Self {
            ambient_intensity: intensity.unwrap_or(0.5),
            uniform: LightUniform::new(),
            // buffer: 
        }
    }
}

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
    pub fn set(&mut self, xyz: glm::Vec3) {
        self.position = xyz
    }
    pub fn update(&mut self)  {
        // Update the light
        unsafe {
            use std::f32::consts::PI;
            static mut INCREMENT: f32 = 0.0;
            INCREMENT += 0.02;
            if INCREMENT > 2.0*PI { INCREMENT = 0.0;}
            self.position = [INCREMENT.cos(), self.position.y, INCREMENT.sin()].into();
        }
    }
    pub fn new() -> Self {
        Self { position: glm::vec3(1.0, 1.0, 1.0), _padding: 0, color: glm::vec3(1.,1.,1.), _padding2: 0 }
    }
}
