use crate::graphics::{GraphicsContext, GraphicsProgram};
use crate::util::print_type_of;
use gl::types::*;
use sdl2::video::GLContext;
use sdl2::video::{GLProfile, Window};
use sdl2::EventPump;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

// yo mama so fat, she sat on the iphone and invented the ipad

pub type GLGraphics = GraphicsContext<GLContext, Window, EventPump, GLuint>;
impl GLGraphics {
    pub fn run_loop(&mut self) {
        (self.preloop_fn)(self);
        while !self.quit_loop {
            (self.input_fn)(self);
            (self.update_fn)(self);
            (self.render_fn)(self);
            if self.backend_initialized {
                self.swap_window();
            }
        }
    }
}
impl GraphicsProgram for GLGraphics {
    unsafe fn create_shader_program(
        &mut self,
        vertex_shader_source: &str,
        frag_shader_source: &str,
    ) {
        unsafe fn compile_shader(shader_type: u32, source: String) -> Result<GLuint, String> {
            // Compile our shaders
            // Based on the type passed in, we create a shader object specifically for that
            // type.
            let shader_object = match shader_type as GLuint {
                gl::VERTEX_SHADER => gl::CreateShader(gl::VERTEX_SHADER),
                gl::FRAGMENT_SHADER => gl::CreateShader(gl::FRAGMENT_SHADER),
                _ => panic!("not a recognized shader type"),
            };
            let len = source.len() as i32;
            let src: *const i8 = CString::new(source.into_bytes()).unwrap().into_raw() as *const i8;
            // The source of our shader
            gl::ShaderSource(shader_object, 1, &src, &len);
            // // Now compile our shader
            gl::CompileShader(shader_object);

            // Retrieve the result of our compilation
            let mut result: GLint = 0;
            // Our goal with glGetShaderiv is to retrieve the compilation status
            gl::GetShaderiv(shader_object, gl::COMPILE_STATUS, &mut result);

            if result == gl::FALSE as i32 {
                let mut length = 0;
                gl::GetShaderiv(shader_object, gl::INFO_LOG_LENGTH, &mut length);
                let error_messages = CString::new(vec![0; length as usize]).unwrap().into_raw();
                gl::GetShaderInfoLog(shader_object, length, &mut length, error_messages);

                let err;
                if shader_type as GLuint == gl::VERTEX_SHADER {
                    err = format!(
                        "ERROR: GL_VERTEX_SHADER compilation failed!\n{:?}",
                        &error_messages
                    );
                    gl::DeleteShader(shader_object);
                    return Err(err);
                } else if shader_type as GLuint == gl::FRAGMENT_SHADER {
                    err = format!(
                        "ERROR: GL_FRAGMENT_SHADER compilation failed!\n{:?}",
                        &error_messages
                    );
                    gl::DeleteShader(shader_object);
                    return Err(err);
                }
            }
            Ok(shader_object)
        }
        let program = gl::CreateProgram();
        let vertex_shader = compile_shader(gl::VERTEX_SHADER, String::from(vertex_shader_source))
            .expect("Failed to compile vertex shader!");
        let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, String::from(frag_shader_source))
            .expect("Failed ot compile fragment shader!");
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        gl::ValidateProgram(program);
        gl::DetachShader(program, vertex_shader);
        gl::DetachShader(program, fragment_shader);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        self.attr_map.insert("PIPELINE".to_string(), program);
    }
    fn swap_window(&self) {
        self.window.gl_swap_window();
    }

    fn get_backend_info(&self) {
        if !self.backend_initialized {
            sdl2::log::log("GL not initialized!");
        } else {
            unsafe {
                sdl2::log::log(&format!(
                    "Vendor: {:?}",
                    CStr::from_ptr(gl::GetString(gl::VENDOR).cast())
                ));
                sdl2::log::log(&format!(
                    "Renderer: {:?}",
                    CStr::from_ptr(gl::GetString(gl::RENDERER).cast())
                ));
                sdl2::log::log(&format!(
                    "Version: {:?}",
                    CStr::from_ptr(gl::GetString(gl::VERSION).cast())
                ));
                sdl2::log::log(&format!(
                    "Shading language: {:?}",
                    CStr::from_ptr(gl::GetString(gl::SHADING_LANGUAGE_VERSION).cast())
                ));
            }
        }
    }
    fn default_state(&self) {
        if self.backend_initialized {
            unsafe {
                gl::Viewport(
                    0,
                    0,
                    self.width.try_into().unwrap(),
                    self.height.try_into().unwrap(),
                );
                gl::Enable(gl::DEPTH_TEST);
                gl::Enable(gl::TEXTURE_2D);
                gl::ClearColor(
                    self.red_channel_bg,
                    self.green_channel_bg,
                    self.blue_channel_bg,
                    self.alpha_channel_bg,
                );
                gl::Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT);
            }
        } else {
            sdl2::log::log("OpenGL not Initialized!");
        }
    }
    fn new(width: u32, height: u32) -> Self {
        let sdl = match sdl2::init() {
            Ok(sdl) => sdl,
            Err(e) => panic!("Failed to initialize SDL!\n{}", sdl2::get_error()),
        };
        let video_subsystem = Box::new(match sdl.video() {
            Ok(video_subsystem) => video_subsystem,
            Err(e) => panic!("Failed to initialize SDL!\n{}", sdl2::get_error()),
        });
        let attrs = video_subsystem.gl_attr();
        attrs.set_context_major_version(4);
        attrs.set_context_minor_version(1);
        attrs.set_context_minor_version(1);
        attrs.set_context_profile(GLProfile::Core);
        attrs.set_double_buffer(true);
        attrs.set_depth_size(24);
        let window = 
            match video_subsystem
                .window("physics-engine", width, height)
                .opengl()
                .build()
            {
                Ok(window) => window,
                Err(e) => panic!("Failed to initialize SDL! {}\n{}", e, sdl2::get_error()),
            };
        let opengl_context = match window.gl_create_context() {
            Ok(opengl_context) => {
                //Initialize function pointers to opengl
                gl::load_with(|s| video_subsystem.gl_get_proc_address(s).cast());
                opengl_context
            }
            Err(e) => panic!("Failed to initialize OpenGL!{}\n{}", e, sdl2::get_error()),
        };
        let event = match sdl.event_pump() {
            Ok(event) => event,
            Err(e) => panic!("Failed to initialize OpenGL!{}\n{}", e, sdl2::get_error()),
        };

        // set function pointers to nop
        fn void_fn(p: &mut GLGraphics) {
            panic!("No callback function set!");
        }
        Self {
            attr_map: HashMap::new(),
            preloop_fn: void_fn,
            input_fn: void_fn,
            update_fn: void_fn,
            render_fn: void_fn,
            width,
            height,
            window,
            event,
            backend: opengl_context,
            quit_loop: false,
            sdl_initialized: true,
            backend_initialized: true,
            red_channel_bg: 0.2,
            blue_channel_bg: 0.2,
            green_channel_bg: 0.2,
            alpha_channel_bg: 0.2,
        }
    }
}
