use physics_engine::geometry::{Mesh, Polyhedron};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::graphics::{GraphicsProgram, GraphicsContext};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let mut camera_uniform = program.initialize_camera();
    let shader_string = include_str!("../shaders/vert.wgsl");
    unsafe {
        // Create pipeline from vertex, fragment shaders
        program.create_shader_program(shader_string);
    }

    program.get_backend_info();

    let mesh = Mesh::from(String::from(include_str!("../assets/bottle.stl")));
    let poly = Polyhedron::from(mesh);

    // Create buffers
    let vertex_buffer = program.create_vertex_buffer(poly.);
    let index_buffer = program.create_index_buffer(indices());
    let camera_buffer = program.create_camera_buffer(&mut camera_uniform);

    program.preloop(&mut |_| {
        println!("Called one time before the loop!");
    });
    event_loop.run(move |event, _, control_flow| {
        match event {
            // INPUT
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == program.window.id() => {
                if program.camera().process_events(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape | VirtualKeyCode::Q),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                //UPDATE
                program.update(&mut |p| {
                    p.camera().update();
                    camera_uniform.update_view_proj(p.camera());
                    p.queue().write_buffer(
                        &camera_buffer,
                        0,
                        bytemuck::cast_slice(&[camera_uniform]),
                    );
                });

                // RENDER
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
                        render_pass.set_pipeline(p.pipeline());
                        render_pass.set_bind_group(0, p.camera_bind_group(), &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..p.n_indices(), 0, 0..1);
                    }

                    // submit will accept anything that implements IntoIter
                    p.queue().submit(std::iter::once(encoder.finish()));
                    output.present();
                });
            }
            Event::MainEventsCleared => program.window.request_redraw(),

            _ => {}
        }
    });
}
fn main() {
    println!("{:#?}", m);
}
