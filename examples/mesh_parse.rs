use std::f32::consts::PI;

use physics_engine::geometry::{
    BoxMesh, CylinderMesh, MeshType, OptimizeMesh, PlaneMesh, Polyhedron, SphereMesh, Transform,
    TriMesh,
};
use physics_engine::shader::CreatePipeline;
use physics_engine::wgpu_program::{MeshBuffer, WGPUGraphics};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");

    use std::env;
    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    // program.get_backend_info();

    // let mut box_mesh = Polyhedron::from(TriMesh::create_box([1.,1.,2.].into()));
    // let mut cylinder_mesh = Polyhedron::from(TriMesh::create_cylinder(1., 2., 30));
    // let mut sphere_mesh = Polyhedron::optimize(TriMesh::create_sphere(1.0, 20, 20));
    // let mut plane_mesh = Polyhedron::from(TriMesh::create_plane());
    let mut mesh = Polyhedron::from(args[1].to_owned());
    // mesh.scale_xyz([0.01,0.01, 0.01].into());
    mesh.set_color([1.0, 0.0, 0.0].into());
    let mesh_transform = Transform::new([0., 0., -1.].into(), [PI / 6., 0., 0.].into());
    // mesh.transform.translate();
    // mesh.transform.rotate_rpy();
    // mesh.update_base();
    // Create buffers
    let mesh_list = vec![mesh];
    let buffer_list: Vec<MeshBuffer> = mesh_list
        .iter()
        .map(|mesh| program.create_mesh_buffer(mesh))
        .collect();

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffers =
        program.create_transform_buffers(mesh_list.iter().map(|m| mesh_transform));
    //
    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    // std::process::exit(0);

    // Create pipeline from vertex, fragment shaders
    let pipeline = program
        .create_render_pipeline(shader_string)
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
                    // p.update_mesh_list(&buffer_list, &mesh_list);
                    p.update_transforms(&transform_buffers, mesh_list.iter().map(|m| mesh_transform));
                });

                // RENDER
                program.render(&mut |p| {
                    p.draw_mesh_list(&pipeline, &buffer_list);
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
