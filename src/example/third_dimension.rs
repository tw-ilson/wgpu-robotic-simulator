use crate::graphics::{Vertex};
use crate::wgpu_program::WGPUGraphics;
// use futures::lock::Mutex;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

fn verts1() -> Vec<Vertex> {
    vec![
        Vertex {
            position: glm::vec3(0.0, 0.5, 0.0),
            color: glm::vec3(1.0, 0.0, 0.0),
            normal: glm::vec3(0.0, 0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(-0.5, -0.5, 0.0),
            color: glm::vec3(0.0, 1.0, 0.0),
            normal: glm::vec3(0.0, 0.0, 0.0),
        },
        Vertex {
            position: glm::vec3(0.5, -0.5, 0.0),
            color: glm::vec3(0.0, 0.0, 1.0),
            normal: glm::vec3(0.0, 0.0, 0.0),
        },
    ]
}

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>, vertices:Vec<Vertex>) {
    // Create buffer
    let vertex_buffer: wgpu::Buffer = program.create_vertex_buffer(vertices);

    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                // (program.update_fn)(&mut program);
                program.update(&mut |_| {});
                program.render(&mut |p| {
                    // -> Result<(), wgpu::SurfaceError>
                    p.set_clear_color((1.0, 1.0, 0.0, 1.0));
                    let output = p
                        .surface()
                        .get_current_texture()
                        .expect("failed to get current texture");
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        p.device()
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            });
                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.1,
                                            g: 0.2,
                                            b: 0.3,
                                            a: 1.0,
                                        }),
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: None,
                            });
                        let pipeline = p.pipeline();
                        render_pass.set_pipeline(pipeline);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..p.n_vert(), 0..1);
                    }

                    // submit will accept anything that implements IntoIter
                    p.queue().submit(std::iter::once(encoder.finish()));
                    output.present();
                });
            }
            Event::MainEventsCleared => {
                program.window.request_redraw()
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }
    });
}

pub fn enter_program() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }
    // let vert_string = include_str!("../../shaders/vert.glsl");
    // let frag_string = include_str!("../../shaders/frag.glsl");
    let shader_string = include_str!("../../shaders/vert.wgsl");

    // let mut sim = physics::ParticleSim::new();
    // sim.setup("");

    let event_loop = winit::event_loop::EventLoop::new();
    let mut program = WGPUGraphics::new(400, 400, &event_loop);

    // Create pipeline from vertex, fragment shaders
    unsafe {
        program.create_shader_program(shader_string);
    }
    let vertices = verts1();
    // program.get_backend_info();
    run_loop(program, event_loop, vertices);
}
