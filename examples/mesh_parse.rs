use std::f32::consts::PI;

use wgpu_robotic_simulator::geometry::{
    BoxMesh, CylinderMesh, MeshType, OptimizeMesh, PlaneMesh, Polyhedron, SphereMesh, Transform,
    TriMesh,
};
use wgpu_robotic_simulator::shader::CreatePipeline;
use wgpu_robotic_simulator::wgpu_program::{MeshBuffer, WGPUGraphics};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
fn run_loop_web(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    } else {
        env_logger::init();
    }
    }

    #[cfg(target_arch = "wasm32")]
{
    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    let _ = window.request_inner_size(PhysicalSize::new(450, 400));
    
    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("wasm-example")?;
            let canvas = web_sys::Element::from(window.canvas()?);
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
    }

 



}

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");

    let args: Vec<String> = std::env::args().collect();
    dbg!(&args);

    let mut mesh = Polyhedron::from(args[1].to_owned());
    mesh.set_color([1.0, 0.0, 0.0].into());
    let mesh_transform = Transform::new(
        [0., 0., -1.].into(),
        [std::f32::consts::PI / 6., 0., 0.].into(),
    );
    let mesh_list = vec![mesh];
    let buffer_list: Vec<MeshBuffer> = mesh_list
        .iter()
        .map(|mesh| program.create_mesh_buffer(mesh))
        .collect();

    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    let transform_buffers =
        program.create_transform_buffers(mesh_list.iter().map(|m| mesh_transform));

    program.create_bindings(&light_buffer, &camera_buffer, &transform_buffers);

    let pipeline = program
        .create_render_pipeline(shader_string)
        .expect("failed to get render pipeline");

    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });
    event_loop.run(move |event, window_target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => {
                match event {
                    WindowEvent::RedrawRequested => {
                        program.update(&mut |p| {
                            p.update_camera(&camera_buffer);
                            p.update_light(&light_buffer);
                            p.update_transforms(
                                &transform_buffers,
                                mesh_list.iter().map(|m| mesh_transform),
                            );
                        });

                        program.render(&mut |p| {
                            p.draw_mesh_list(&pipeline, &buffer_list);
                        });
                    }
                    WindowEvent::CloseRequested => window_target.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => window_target.exit(),
                    _ => {}
                }
                if program.process_keyboard(event) {}
            }
            Event::DeviceEvent { event, .. } => {
                // program.mouse_look(event.delta.0 as f32, 0.0)
            }
            _ => {}
        }
    });
}
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let program = WGPUGraphics::new(1240, 860, &event_loop);

    run_loop(program, event_loop);
}
