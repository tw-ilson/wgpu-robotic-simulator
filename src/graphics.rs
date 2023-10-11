extern crate gl;
extern crate sdl2;
use gl::types::*;
use std::collections::HashMap;
use std::sync::Arc;
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[derive(Clone)]
pub struct GraphicsContext
// Backend, Window, Event, Resource
<B, W, E, R> {
    // map runtime attribute names to pointers
    pub attr_map: HashMap<String, R>,

    // function pointers called in main program loop
    pub preloop_fn: fn(&mut GraphicsContext<B, W, E, R>),
    pub input_fn: fn(&mut GraphicsContext<B, W, E, R>),
    pub update_fn: fn(&mut GraphicsContext<B, W, E, R>),
    pub render_fn: fn(&mut GraphicsContext<B, W, E, R>),

    // window
    pub width: u32,
    pub height: u32,
    pub backend: Box<B>,
    pub window: Box<W>,
    pub event: Box<E>,

    // flags
    pub quit_loop: bool,
    pub sdl_initialized: bool,
    pub backend_initialized: bool,

    pub red_channel_bg: f32,
    pub blue_channel_bg: f32,
    pub green_channel_bg: f32,
    pub alpha_channel_bg: f32,
}

impl<B, W, E, R> GraphicsContext<B, W, E, R>
where
    GraphicsContext<B, W, E, R>: GraphicsProgram,
{
    pub fn quit(&mut self) {
        self.quit_loop = true;
    }
    pub fn run_loop(&mut self) {
        (self.preloop_fn)(self);
        while !self.quit_loop {
            (self.input_fn)(self);
            (self.update_fn)(self);
            (self.render_fn)(self);
            if self.backend_initialized {
                self.swap_window();
            }
        }
    }
    pub fn set_clear_color(&mut self, (r, g, b, a): (f32, f32, f32, f32)) {
        self.red_channel_bg = r;
        self.green_channel_bg = g;
        self.blue_channel_bg = b;
        self.alpha_channel_bg = a;
    }
}

pub trait GraphicsProgram {
    // unsafe fn compile_shader(shader_type:u32, source:&String) -> Result<GLuint, String>;
    unsafe fn create_shader_program(
        &mut self,
        vertex_shader_source: &str,
        frag_shader_source: &str,
    );
    fn swap_window(&self);
    fn get_backend_info(&self);
    fn default_state(&self);
    fn new(width: u32, height: u32) -> Self;
}
