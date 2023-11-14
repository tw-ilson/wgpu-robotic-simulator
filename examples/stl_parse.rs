use physics_engine::geometry::{Mesh, MeshType, Polyhedron};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::graphics::{GraphicsProgram, GraphicsContext};
use physics_engine::shader::create_shader_program;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    let shader_string = include_str!("../shaders/shader.wgsl");
    // let light_shader_string = include_str!("../shaders/light.wgsl");
    
    // Create pipeline from vertex, fragment shaders
    let pipeline = unsafe { create_shader_program(&program, shader_string) };
    // let light_pipeline = unsafe { create_shader_program(&program, light_shader_string) };

    program.get_backend_info();

    let mesh = Mesh::from(MeshType::STL(String::from(include_str!("../assets/teapot-converted-ASCII.stl"))));
    let mut poly = Polyhedron::from(mesh);
    poly.scale(0.5);

    // Create buffers
    let vertex_buffer = program.create_vertex_buffer(poly.verts);
    let index_buffer = program.create_index_buffer(poly.indices);

    //Initialize uniform buffers
    let camera_buffer = program.create_camera_buffer();
    let light_buffer = program.create_light_buffer();
    program.create_bind_groups(&[&camera_buffer, &light_buffer]);

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
                if program.process_keyboard(event) {
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
                        _ => {}
                    }
                }
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
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        // render_pass.set_pipeline(&light_pipeline);
                        // render_pass.draw_light_model
                        render_pass.set_pipeline(&pipeline);
                        render_pass.set_bind_group(0, p.camera_bind_group(), &[]);
                        render_pass.set_bind_group(1, p.light_bind_group(), &[]);
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
    let event_loop = winit::event_loop::EventLoop::new();
    let program = WGPUGraphics::new(800, 600, &event_loop);

    run_loop(program, event_loop);
}
