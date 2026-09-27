use std::f32::consts::PI;

use std::str::FromStr;
use wgpu_robotic_simulator::bindings::*;
use wgpu_robotic_simulator::geometry::{BoxMesh, CylinderMesh, Polyhedron, TriMesh};
use wgpu_robotic_simulator::graphics::GraphicsProgram;
use wgpu_robotic_simulator::robot::RobotGraphics;
use wgpu_robotic_simulator::shader::CreatePipeline;
use wgpu_robotic_simulator::urdf::*;
use wgpu_robotic_simulator::wgpu_program::{MeshBuffer, WGPUGraphics};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

async fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = winit::window::Window::new(&event_loop)?;
    let mut program = WGPUGraphics::new(1240, 860, &window).await;
    program.get_backend_info();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        // (The canvas is attached to #wasm-example by WGPUGraphics::new.)
        use winit::dpi::PhysicalSize;
        let _ = program.window.request_inner_size(PhysicalSize::new(1240, 860));
    }

    let mut robot = RobotDescriptor::from_str(include_str!("../assets/xarm.urdf"))
        .expect("unable to read urdf");

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffers = program.robot_create_transform_buffers(&robot);
    let mesh_buffers = program.robot_create_mesh_buffers(&robot);
    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    // Create pipeline from vertex, fragment shaders
    let pipeline = program
        .create_render_pipeline(include_str!("../shaders/shader.wgsl"))
        .expect("failed to get render pipeline!");

    let mut increment = 0.0;
    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });
    // Kick off the first redraw explicitly. On web (winit) no initial
    // RedrawRequested is delivered, so without this the render loop
    // below would never start (black canvas).
    program.window.request_redraw();
    event_loop.run(move |event, control_flow| {
        match event {
            // INPUT
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        KeyCode::Escape | KeyCode::KeyQ => control_flow.exit(),
                        keycode => {
                            program.process_keyboard(keycode);
                        }
                    },
                    WindowEvent::RedrawRequested => {
                        program.window.request_redraw();
                        //UPDATE
                        program.update(&mut |p| {
                            p.update_camera(&camera_buffer);
                            p.update_light(&light_buffer);
                            increment = (increment + 0.02) % (2.0 * PI);
                            robot.set_joint_position(
                                &[
                                    0.,
                                    0.,
                                    increment.cos(),
                                    -increment.cos(),
                                    -increment.cos(),
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                    0.,
                                ],
                                false,
                            );
                            robot.build();
                            p.robot_assign_transform_buffers(&robot, &transform_buffers);
                        });

                        // RENDER
                        program.render(&mut |p| {
                            p.draw_robot(&robot, &mesh_buffers, &pipeline);
                        });
                    }
                    // WindowEvent::DeviceEvent {
                    //     event: DeviceEvent::MouseMotion{ delta, },
                    //     .. // We're not using device_id currently
                    // } =>  {
                    //     // program.mouse_look(
                    //     //     delta.0 as f32,
                    //     //     0.0
                    //     //     // delta.1 as f32
                    //     //     )
                    // },
                    _ => {}
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}

pub fn main() {
    start();
}

/// Entry point for both native and web.
///
/// On web this is invoked automatically by wasm-bindgen on module load
/// (`#[wasm_bindgen(start)]`). GPU setup is async: the browser event loop
/// drives it via `spawn_local`, while native blocks on it.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            wasm_bindgen_futures::spawn_local(run_until_error());
        } else {
            env_logger::init();
            futures::executor::block_on(run_until_error());
        }
    }
}

async fn run_until_error() {
    if let Err(e) = run().await {
        log::error!("run failed: {e:?}");
    }
}
