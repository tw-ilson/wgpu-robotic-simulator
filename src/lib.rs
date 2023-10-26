pub mod graphics;
// pub mod opengl_program;
pub mod camera;
pub mod geomety;
pub mod physics;
pub mod util;
pub mod wgpu_program;
extern crate nalgebra_glm as glm;
extern crate env_logger;

// use util::print_type_of;
use graphics::{GraphicsContext, GraphicsProgram};
// use opengl_program::*;
use physics::*;
use wgpu_program::*;
