use std::f32::consts::PI;

use physics_engine::geometry::{Polyhedron, TriMesh, BoxMesh, CylinderMesh, MeshBuffer, PlaneMesh};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::graphics::GraphicsProgram;
use physics_engine::shader::CompilePipeline;
use physics_engine::bindings::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");

    program.get_backend_info();

    let mut box_mesh = Polyhedron::from(TriMesh::create_box([1.,1.,2.].into()));
    let mut cylinder_mesh = Polyhedron::from(TriMesh::create_cylinder(1., 2., 30));
    let mut plane_mesh = Polyhedron::from(TriMesh::create_plane());
    // plane_mesh.set_color([0.0, 1.0, 0.0].into());
    // box_mesh.transform.rotate_rpy([PI/4., 0., 0.].into());
    cylinder_mesh.transform.rotate_rpy([-1.5708, 0.,0.].into());
    // box_mesh.update_base();
    cylinder_mesh.update_base();
    // let mesh = Polyhedron::from("assets/mesh/teapot.stl".to_owned());

    // Create buffers
    let mesh_list = vec![cylinder_mesh, plane_mesh];
    let buffer_list:Vec<MeshBuffer> = mesh_list.iter().map(|mesh| program.create_mesh_buffer(mesh)).collect();

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffer = program.create_transform_buffer(&mesh_list);

    program.new_bind_group_layout("camera_bind_group", &[uniform_layout_entry()]);
    program.new_bind_group_layout("light_bind_group", &[uniform_layout_entry()]);
    program.new_bind_group_layout("transform_bind_group", &[uniform_layout_entry()]);
    program.create_bind_groups(&[
                               &camera_buffer,
                               &light_buffer,
                               &transform_buffer,
    ]);

    
    // Create pipeline from vertex, fragment shaders
    let pipeline = program.create_shader_program(shader_string);

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
                program.mouse_look(
                    delta.0 as f32,
                    0.0
                    // delta.1 as f32
                    )
            },
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                //UPDATE
                program.update(&mut |p| {
                    p.update_camera(&camera_buffer);
                    p.update_light(&light_buffer);
                    // p.update_mesh_list(&buffer_list, &mesh_list);
                });

                // RENDER
                program.render(&mut |p| {
                    p.draw_mesh_list(&pipeline, &buffer_list, &mesh_list, &camera_buffer, &light_buffer, &transform_buffer);
                    // submit will accept anything that implements IntoIter
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
