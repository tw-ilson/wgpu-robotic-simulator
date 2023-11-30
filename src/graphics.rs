use std::collections::HashMap;
// use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub color: glm::Vec3,
    pub normal: glm::Vec3,
}
impl From<glm::Vec3> for Vertex {
    fn from(position:glm::Vec3) -> Self {
        Self { position, color: glm::vec3(1.,1.,1.), normal: glm::vec3(0., 0., 0.) }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ContextFlags {
    pub quit_loop: bool,
    pub sdl_initialized: bool,
    pub backend_initialized: bool,
}

#[derive(Clone)]
pub struct GraphicsContext<B, W, R> {
    // map runtime attribute names to pointers
    pub attr_map: HashMap<String, R>,

    // window
    pub width: u32,
    pub height: u32,
    pub backend: B,
    pub window: W,
    // pub event: &E,

    // flags
    pub flags: ContextFlags,

    pub bg_color: Color,
}

impl<B, W, R> GraphicsContext<B, W, R>
where
    GraphicsContext<B, W, R>: GraphicsProgram,
{
    pub fn quit(&mut self) {
        self.flags.quit_loop = true;
    }
    pub fn set_clear_color(&mut self, (r, g, b, a): (f32, f32, f32, f32)) {
        self.bg_color = Color { r, g, b, a };
    }
    // why am I doing this?
    // closures capture calling scope
    pub fn preloop<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut GraphicsContext<B, W, R>),
    {
        f(self)
    }
    pub fn input<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut GraphicsContext<B, W, R>),
    {
        f(self)
    }
    pub fn update<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut GraphicsContext<B, W, R>),
    {
        f(self)
    }
    pub fn render<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut GraphicsContext<B, W, R>),
    {
        f(self)
    }
}

pub trait GraphicsProgram {
    fn swap_window(&self);
    fn get_backend_info(&self);
    fn default_state(&mut self);
}
