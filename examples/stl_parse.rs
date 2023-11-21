use physics_engine::geometry::{Polyhedron, TriMesh, BoxMesh, CylinderMesh, Transform};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::graphics::{GraphicsProgram};
use physics_engine::shader::create_shader_program;
use nalgebra_glm as glm;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");

    program.get_backend_info();

    // let mut mesh = Mesh::from(MeshType::STL(String::from(include_str!())));
    let box_mesh = Polyhedron::from(TriMesh::create_box([1.,1.,1.].into()));
    let cylinder_mesh = Polyhedron::from(TriMesh::create_cylinder(1., 2., 30));
    // box_mesh.transform;

    // let mut poly = Polyhedron::from_file("../assets/mesh/stl.stl");
    // poly.calculate_normals();
    // poly.rotate(-glm::pi::<f32>()/2., [1.,0.,0.].into());

    // Create buffers
    let vao_list = vec![
        program.create_vao(cylinder_mesh),
        program.create_vao(box_mesh), 
    ];

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffer = program.create_buffer("transform_buffer" , &[Transform::default()], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
    program.create_bind_groups(&[
                               // &transform_buffer,
                               &camera_buffer,
                               &light_buffer,
    ]);

    println!("{:#?}", program.bind_layouts());
    println!("{:#?}", program.bind_groups());
    
    // Create pipeline from vertex, fragment shaders
    let pipeline = unsafe { create_shader_program(&program, shader_string) };

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
                });

                // RENDER
                program.render(&mut |p| {
                    p.draw(&pipeline, &vao_list);

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
    let program = WGPUGraphics::new(1800, 1600, &event_loop);

    run_loop(program, event_loop);
}
