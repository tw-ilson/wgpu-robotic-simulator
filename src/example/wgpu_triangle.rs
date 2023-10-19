use crate::graphics::GraphicsProgram;
use crate::wgpu_program::WGPUGraphics;
// use futures::lock::Mutex;
use std::sync::{Arc, Mutex};
use winit::{
    event::*,
    event_loop::{self, ControlFlow, EventLoop},
};

fn preloop(program: &mut WGPUGraphics) {
    println!("Called one time before the loop!");
    program.default_state();
}

fn update(program: &mut WGPUGraphics) {}
fn input(program: &mut WGPUGraphics) {}
fn render(program: &mut WGPUGraphics)
// -> Result<(), wgpu::SurfaceError>
{
    program.set_clear_color((1.0, 1.0, 0.0, 1.0));
    let output = program
        .surface()
        .get_current_texture()
        .expect("failed to get current texture");
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = program
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
    {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    }

    // submit will accept anything that implements IntoIter
    program.queue().submit(std::iter::once(encoder.finish()));
    output.present();
    // Ok(())
}

fn vertex_specification() -> u32 {
    0
}
pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    program.preloop(&mut |p| {
        println!("Called one time before the loop!");
        p.default_state();
    });
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                // (program.update_fn)(&mut program);
                program.update(&mut |p| {});

                program.render(&mut |p| {
                    p.set_clear_color((1.0, 1.0, 0.0, 1.0));
                    let output = p
                        .surface()
                        .get_current_texture()
                        .expect("failed to get current texture");
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        p
                            .device()
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            });
                    {
                        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    }

                    // submit will accept anything that implements IntoIter
                    p.queue().submit(std::iter::once(encoder.finish()));
                    output.present();
                    // Ok(())
                });
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
    let vert_string = include_str!("../../shaders/vert.glsl");
    let frag_string = include_str!("../../shaders/frag.glsl");
    // let mut sim = physics::ParticleSim::new();
    // sim.setup("");
    let event_loop = winit::event_loop::EventLoop::new();
    let mut program = WGPUGraphics::new(400, 400, &event_loop);

    // Create pipeline from vertex, fragment shaders
    // unsafe { program.create_shader_program(vert_string, frag_string); }

    // Create buffers from vertex specification
    // unsafe { vertex_specification(&mut program); }

    // program.get_backend_info();
    run_loop(program, event_loop);
}
