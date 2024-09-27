use nalgebra_glm as glm;
use physics_engine::graphics::GraphicsProgram;
use physics_engine::opengl_program::GLGraphics;

unsafe fn vertex_specification(program: &mut GLGraphics) {
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
        vertex_data.as_mut_ptr() as *const _, // Raw array of data
        gl::STATIC_DRAW,
    ); // How we intend to use the data

    let posn_stride = (size_of::<f32>() * 6) as i32;
    let color_stride = posn_stride;
    let posn_offset = 0 as *const c_void;
    let color_offset = (size_of::<f32>() * 3) as *const c_void;

    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(
        0,         // Attribute 0 corresponds to the enabled gl::EnableVertexAttribArray
        3,         // The number of components (e.g. x,y,z = 3 components)
        gl::FLOAT, // Type
        gl::FALSE, // Is the data normalized
        posn_stride,
        posn_offset, // Offset
    );

    gl::EnableVertexAttribArray(1);
    gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, color_stride, color_offset);

    gl::BindVertexArray(0);
    gl::DisableVertexAttribArray(0);
    gl::DisableVertexAttribArray(1);

    program
        .attr_map
        .insert("VAO".to_string(), vertex_array_object);
    program
        .attr_map
        .insert("VBO".to_string(), vertex_buffer_object);
}
pub fn run_loop(mut program: GLGraphics) {
    program.preloop(&mut |program| {
        println!("Called one time before the loop!");
    });
    let mut event_pump = program.sdl().event_pump().unwrap();
    while !program.flags.quit_loop {
        program.input(&mut |program| {
            use sdl2::event::Event;
            use sdl2::keyboard::Scancode;
            let ep = event_pump.poll_iter();
            for event in ep {
                if let Event::Quit { .. } = event {
                    return program.quit();
                }
            }
        });
        program.update(&mut |program| {});
        program.render(&mut |program| {
            // program.default_state();
            // Enable our attributes
            program.set_clear_color((1.0, 1.0, 0.0, 1.0));
            program.default_state();
            unsafe {
                let pipeline = *program
                    .attr_map
                    .get("PIPELINE")
                    .expect("Nothing bound to OpenGL Pipeline Program");
                let vao = *program
                    .attr_map
                    .get("VAO")
                    .expect("Nothing bound to Vertex Array Object");
                let vbo = *program
                    .attr_map
                    .get("VBO")
                    .expect("Nothing bound to Vertex Buffer Object");

                gl::UseProgram(pipeline);
                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
            }
            if program.flags.backend_initialized {
                program.swap_window();
            }
        });
    }
}
pub fn enter_program() {
    let vert_string = include_str!("../shaders/vert.glsl");
    let frag_string = include_str!("../shaders/frag.glsl");
    // let mut sim = physics::ParticleSim::new();
    // sim.setup("");
    let mut program = GLGraphics::new(600, 600);

    // Create pipeline from vertex, fragment shaders
    unsafe {
        program.create_shader_program(vert_string, frag_string);
    }

    // Create buffers from vertex specification
    unsafe {
        vertex_specification(&mut program);
    }

    // program.get_backend_info();
    run_loop(program);
}
fn main() {
    enter_program()
}
