mod graphics;
mod opengl_program;
mod physics;
mod util;
mod wgpu_program;
mod example;

// use util::print_type_of;
use graphics::{GraphicsContext, GraphicsProgram};
use opengl_program::*;
use physics::*;
use wgpu_program::*;


fn main() {
    example::sdl_gl_triangle::enter_program()
}
