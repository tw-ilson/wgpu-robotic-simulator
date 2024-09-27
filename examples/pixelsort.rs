use itertools::Itertools;
use physics_engine::shader::CreatePipeline;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// use std::log;
use physics_engine::graphics::{GraphicsProgram, Vertex};
use physics_engine::wgpu_program::WGPUGraphics;
use physics_engine::texture::Texture;
// use futures::lock::Mutex;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

const SHADER_STRING: &str = "
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
";


pub fn run_loop(mut program: WGPUGraphics, event_loop: EventLoop<()>) {
    // Create buffer
    let pipeline = program
        .create_render_pipeline(SHADER_STRING)
        .expect("failed to get render pipeline!");

    let (vertices, indices): (Vec<Vertex>, Vec<u32>) = 
    (
        vec![
            [-1., 1., 0.].into(),
            [1., 1., 0.].into(),
            [-1., -1., 0.].into(),
            [1., -1., 0.].into(),
        ],
        vec![
            2,1,0,
            1,2,3
        ],
    );
    let vertex_buffer: wgpu::Buffer = program.create_vertex_buffer(&vertices);
    let index_buffer: wgpu::Buffer = program.create_index_buffer(&indices);

    let diffuse_bytes = include_bytes!("assets/image/flower.jpg");
    let image_texture = Texture::from_bytes(program.device(), program.queue(), diffuse_bytes, "image");

    program.preloop(&mut |p| {
        println!("Called one time before the loop!");
    });
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(window_id) if window_id == program.window.id() => {
                // program.update(&mut |p| { p.default_state() });
                program.render(&mut |p| {
                    // -> Result<(), wgpu::SurfaceError>
                    p.set_clear_color((1.0, 1.0, 0.0, 1.0));
                    let output = p.surface().get_current_texture().unwrap();
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
                        p.assign_texture(image_texture_buffer, )
                        render_pass.set_pipeline(&pipeline);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                    }

                    // submit will accept anything that implements IntoIter
                    p.queue().submit(std::iter::once(encoder.finish()));
                    output.present();
                });
            }
            Event::MainEventsCleared => program.window.request_redraw(),

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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn enter_program() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = winit::event_loop::EventLoop::new();
    let mut program = WGPUGraphics::new(800, 600, &event_loop);

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        program.window.set_inner_size(PhysicalSize::new(800, 600));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(program.window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // Create pipeline from vertex, fragment shaders
    // program.get_backend_info();
    run_loop(program, event_loop);
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(main))]
pub fn main() {
    enter_program()
}
