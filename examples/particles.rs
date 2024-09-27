use physics_engine::urdf::RobotGraphics;
use physics_engine::{geometry::*, light};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::physics::*;
use physics_engine::shader::CreatePipeline;

use nalgebra_glm as glm;
use rand::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::{thread, time};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
struct Object {
    pub dynamics: FreeBody,
    pub geometry: Polyhedron,
    pub transform: Transform,
}

type Particle = Object;

impl Distribution<Particle> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle {
        let dynamics = FreeBody {
                posn: glm::vec3(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.0),
                vel: glm::Vec3::zeros(),
                force: glm::Vec3::zeros(),
                // accel: glm::vec2(0, 0),
                mass: 1.0,
                theta: rng.gen_range(0.0..2.*PI),
                omega: 0.0,
                // alpha: 
            };
        Particle {
            dynamics,
            geometry: Polyhedron::from(TriMesh::create_sphere(0.01, 20, 20)),
            transform: Transform::default()
        }
    }
}

impl Particle {
    fn compute_gravity(&self) -> glm::Vec3 {
        glm::vec3(0.0, 0.0, -9.8 * self.dynamics.mass)
    }
}

#[derive(Clone, Debug)]
pub struct ParticleSim(Vec<Particle>);

impl PhysicsProgram for ParticleSim {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn setup(&mut self, scene: &str) {
        let mut rng = rand::thread_rng();
        let ParticleSim(particles) = self;
        let n = 2;
        for _ in 0..n {
            let p: Particle = rng.gen();
            particles.push(p);
        }
    }
    fn step(&mut self) {
        let ParticleSim(particles) = self;
        let dt = 1;

        for p in particles.iter_mut() {
            let fg = p.compute_gravity();
            let a = fg / p.dynamics.mass;
            // p.dynamics.vel += a * (dt as f32);
            // p.dynamics.posn += p.dynamics.vel * (dt as f32);
            p.transform = Transform::new(p.dynamics.posn, glm::Vec3::zeros());
            // dbg!(p.transform);
        } 
    }
    fn apply_forces(&mut self) {}
    fn update_kinematics(&mut self) {}
    fn detect_collisions(&mut self) {}
    fn solve_constraints(&mut self) {}
}

const SHADER_STRING: &str = "
struct Transform {
    tmatrix: mat4x4<f32>,
}
@group(2) @binding(0)
var<uniform> transform: Transform;

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = transform.tmatrix * vec4<f32>(model.position + vec3(0.0,0.3,0.0), 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
";
fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let mut sim = ParticleSim::new();
    sim.setup("scene1");

    let transform_buffers = program.create_transform_buffers(sim.0.iter().map(|p| p.transform));
    let light_buffer = program.create_light_buffer();
    let camera_buffer = program.create_camera_buffer();
    let mesh_buffers = program.create_mesh_buffers(sim.0.iter().map(|p|&p.geometry));
    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    let pipeline = program
        .create_render_pipeline(SHADER_STRING)
        .expect("failed to get render pipeline");

    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            // INPUT
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => {
                match event {
                    WindowEvent::CloseRequested
                     => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => if input.state == ElementState::Pressed {*control_flow = ControlFlow::Exit},
                            Some(VirtualKeyCode::Q) => if input.state == ElementState::Pressed {*control_flow = ControlFlow::Exit},
                            _ => {}
                        }
                    }
                    _ => {},
                }
                if program.process_keyboard(event){}
            },
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                //UPDATE
                program.update(&mut |p| {
                    sim.step();
                    p.update_transforms(&transform_buffers, sim.0.iter().map(|p|&p.transform));
                });

                // RENDER
                program.render(&mut |p| {
                    p.draw_mesh_list(&pipeline, &mesh_buffers);
                    // submit will accept anything that implements IntoIter
                });
            }
            Event::MainEventsCleared => program.window.request_redraw(),
            _ => {}
        }
    });
}
pub fn enter_program() {

    // let mut sim = physics::ParticleSim::new();
    let event_loop = winit::event_loop::EventLoop::new();
    let mut program = WGPUGraphics::new(600, 600, &event_loop);

    // program.get_backend_info();
    run_loop(program, event_loop);
}

fn main() {
    enter_program()
}
