use crate::opengl_program::GLGraphics;
use crate::physics;
unsafe fn vertex_specification(program: &mut GLGraphics) {
    let MAX_PARTICLES = 300;
    use std::ffi::c_void;
    use std::mem::size_of;

    let mut vertex_buffer: u32 = 0;
    let mut particle_posn_buffer: u32 = 0;
    let mut particle_color_buffer: u32 = 0;

    let mut vertex_data: Vec<f32> = vec![
         -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
         -0.5, 0.5, 0.0,
         0.5, 0.5, 0.0,
    ];

    gl::GenBuffers(1, &mut vertex_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
    gl::BufferData(
        gl::ARRAY_BUFFER, // Kind of buffer we are working with
        (vertex_data.len() * size_of::<f32>()) as isize, // Size of data in bytes
        vertex_data.as_mut_ptr() as *const _, // Raw array of data
        gl::STATIC_DRAW);

    gl::GenBuffers(1, &mut particle_posn_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, particle_posn_buffer);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        MAX_PARTICLES * 4 * size_of::<f32>() as isize,
        std::ptr::null(), //this will be updated each frame
        gl::STREAM_DRAW);
    
    gl::GenBuffers(1, &mut particle_color_buffer);
    gl::BindBuffer(gl::ARRAY_BUFFER, particle_color_buffer);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        MAX_PARTICLES * 4 * size_of::<f32>() as isize,
        std::ptr::null(),
        gl::STREAM_DRAW);

    // program
    //     .attr_map
    //     .insert("VAO".to_string(), vertex_array_object);
    program
        .attr_map
        .insert("VBO".to_string(), vertex_buffer);
}
fn run_loop(mut program: GLGraphics) {
    program.preloop(|program| {
        println!("Called one time before the loop!");
    });
    while !program.quit_loop {
        program.input(|program| {
            use sdl2::event::Event;
            use sdl2::keyboard::Scancode;
            let ep = program.event.poll_iter();
            for event in ep {
                if let Event::Quit { .. } = event {
                    return program.quit();
                }
            }
        });
        program.update(|program| {});
        program.render(|program| {
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
            if program.backend_initialized {
                program.swap_window();
            }
        });
    }
}
pub fn enter_program() {
    let vert_string = include_str!("../../shaders/vert.glsl");
    let frag_string = include_str!("../../shaders/frag.glsl");
    let mut sim = physics::ParticleSim::new();
    sim.setup("");
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
