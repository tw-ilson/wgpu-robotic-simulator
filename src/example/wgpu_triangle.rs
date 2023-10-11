use crate::wgpu_program::WGPUGraphics;
use crate::graphics::GraphicsProgram;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

fn preloop(program: &mut WGPUGraphics) {
    println!("Called one time before the loop!");
    program.default_state();
}

fn update(program: &mut WGPUGraphics) { }
fn input(program: &mut WGPUGraphics) { 
    program.event.run(|event, _, control_flow| match event {
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
    });
}
fn render(program: &mut WGPUGraphics) { 
    program.set_clear_color((1.0, 1.0, 0.0, 1.0));
    let output = program.surface().get_current_texture()
        .expect("failed to get current texture");
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = program.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
}

pub fn enter_program() { 
    let vert_string = include_str!("../../shaders/vert.glsl");
    let frag_string = include_str!("../../shaders/frag.glsl");
    // let mut sim = physics::ParticleSim::new();
    // sim.setup("");
    let mut program = WGPUGraphics::new(400, 400);

    // set callbacks
    program.preloop_fn = preloop; program.input_fn = input; program.update_fn = update; program.render_fn = render;

    // Create pipeline from vertex, fragment shaders
    unsafe { program.create_shader_program(vert_string, frag_string); }

    // Create buffers from vertex specification
    // unsafe { vertex_specification(&mut program); }

    // program.get_backend_info();
    program.run_loop();
}
