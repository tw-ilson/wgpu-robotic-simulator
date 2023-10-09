mod graphics;
mod opengl_program;
mod wgpu_program;
mod physics;
mod util;

// use util::print_type_of;
use graphics::{GraphicsContext, GraphicsProgram};
use opengl_program::*;
use wgpu_program::*;
use physics::*;

use crate::util::print_type_of;

fn preloop(program: &mut GLGraphics) {
    println!("Called one time before the loop!");
    // program.default_state()
}
fn input(program: &mut GLGraphics) {
    use sdl2::event::Event;
    use sdl2::keyboard::Scancode;
    let ep = program.event.poll_iter();
    for event in ep {
        if let Event::Quit { .. } = event {
            return program.quit();
        }
    }
    // let mut color = (0.5, 0.0, 0.0, 1.0);
    // for scancode in program.event.keyboard_state().pressed_scancodes() {
    //     match scancode {
    //         Scancode::Num1 => color = (1., 0., 0., 1.),
    //         Scancode::Num2 => color = (0., 1., 0., 1.),
    //         Scancode::Num3 => color = (0., 0., 1., 1.),
    //         _ => {}
    //     }
    // }
    // program.set_clear_color(color);
}

fn update(program: &mut GLGraphics) {}

fn render(program: &mut GLGraphics) {
    // program.default_state();
    // Enable our attributes
    program.set_clear_color((1.0, 1.0, 0.0, 1.0));
    program.default_state();
    unsafe {
        let pipeline = *program.attr_map.get("PIPELINE")
            .expect("Nothing bound to OpenGL Pipeline Program");
        let vao = *program
            .attr_map
            .get("VAO")
            .expect("Nothing bound to Vertex Array Object");
        let vbo = *program
            .attr_map
            .get("VBO")
            .expect("Nothing bound to Vertex Buffer Object");

        // println!("After: {}, {}", vao, vbo);
        gl::UseProgram(pipeline);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
}
unsafe fn vertex_specification() -> (u32, u32) {
    use std::ffi::c_void;
    use std::mem::size_of;
    let mut vertex_array_object: u32 = 0;
    let mut vertex_buffer_object: u32 = 0;
    let mut vertex_data: Vec<f32> = vec![
        -0.8, -0.8, 0.0, // Let vertex position
        1.0, 0.0, 0.0, // Let vertex color
        0.8, -0.8, 0.0, // right vertex position
        0.0, 1.0, 0.0, // right vertex color
        0.0, 0.8, 0.0, // Top vertex position
        0.0, 0.0, 1.0, // Top vertex color
    ];
    gl::GenVertexArrays(1, &mut vertex_array_object);
    gl::BindVertexArray(vertex_array_object);

    gl::GenBuffers(1, &mut vertex_buffer_object);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_object);
    gl::BufferData(
        gl::ARRAY_BUFFER, // Kind of buffer we are working with
        (vertex_data.len() * size_of::<f32>()) as isize, // Size of data in bytes
        vertex_data.as_mut_ptr() as *const _,       // Raw array of data
        gl::STATIC_DRAW,
    ); // How we intend to use the data

    let posn_stride = (size_of::<f32>() * 6) as i32;
    let color_stride = posn_stride;
    let posn_offset = 0 as *const c_void;
    let color_offset = (size_of::<f32>() * 3) as *const c_void;

    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0,           // Attribute 0 corresponds to the enabled gl::EnableVertexAttribArray
        3,           // The number of components (e.g. x,y,z = 3 components)
        gl::FLOAT,   // Type
        gl::FALSE,   // Is the data normalized
        posn_stride,
        posn_offset, // Offset
    );

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, color_stride, color_offset);

    gl::BindVertexArray(0);
    gl::DisableVertexAttribArray(0);
    gl::DisableVertexAttribArray(1);

    (vertex_array_object, vertex_buffer_object)
}
fn enter_program() {
    let vert_string = r#"
    #version 330 core
    layout(location=0)in vec3 position; 
    layout(location=1)in vec3 vertexColor;
    out vec3 theColor;
    void main()
    {
      gl_Position = vec4(position.x, position.y, position.z, 1.0f);
      theColor = vertexColor;
    }
    "#;
    let frag_string = r#"
    #version 330 core
    in vec3 theColor;
    out vec4 color;
    void main()
    {
      color = vec4(theColor, 1.0);
    }
    "#;
    // let mut sim = physics::ParticleSim::new();
    // sim.setup("");
    let mut program = GLGraphics::new(600, 600);
    program.preloop_fn = preloop;
    program.input_fn = input;
    program.update_fn = update;
    program.render_fn = render;

    let (vao, vbo);
    let shader_program;
    unsafe {
        shader_program = GLGraphics::create_shader_program(vert_string, frag_string);
        (vao, vbo) = vertex_specification();
    };

    program.attr_map.insert("PIPELINE".to_string(), shader_program);
    program.attr_map.insert("VAO".to_string(), vao);
    program.attr_map.insert("VBO".to_string(), vbo);

    // program.get_backend_info();
    program.run_loop()
}

fn main() {
    enter_program()
}
