use physics_engine::graphics::GraphicsProgram;
use physics_engine::shader::CreatePipeline;
use physics_engine::urdf::*;
use physics_engine::wgpu_program::WGPUGraphics;
use std::f32::consts::PI;
use std::str::FromStr;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");

    program.get_backend_info();

    // NOT WORKING !!
    let mut robot = RobotDescriptor::from_str(include_str!("../assets/LittleDog.urdf"))
        .expect("unable to read urdf");

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffers = program.robot_create_transform_buffers(&robot);
    let mesh_buffers = program.robot_create_mesh_buffers(&robot);
    //
    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    // Create pipeline from vertex, fragment shaders
    let pipeline = program
        .create_render_pipeline(shader_string)
        .expect("unable to create render pipeline");

    let mut increment = 0.0;
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
                            // Some(VirtualKeyCode::Q) => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    _ => {},
                }
                if program.process_keyboard(event){}
                
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } =>  {
                // program.mouse_look(
                //     delta.0 as f32,
                //     0.0
                //     // delta.1 as f32
                //     )
            },
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                //UPDATE
                program.update(&mut |p| {
                    p.update_camera(&camera_buffer);
                    p.update_light(&light_buffer);
                    increment = (increment + 0.02) % (2.0*PI);
                    robot.set_joint_position(&[0.,0.,increment.cos(),-increment.cos(),-increment.cos(),0.,0.,0.,0.,0.,0.,0.,], false);
                    robot.build();
                    p.robot_assign_transform_buffers(&robot, &transform_buffers);
                });

                // RENDER
                program.render(&mut |p| {
                    p.draw_robot(&robot, &mesh_buffers, &pipeline);
                });
            }
            Event::MainEventsCleared => program.window.request_redraw(),
            _ => {}
        }
    });
}
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let program = WGPUGraphics::new(1240, 860, &event_loop);

    run_loop(program, event_loop);
}
