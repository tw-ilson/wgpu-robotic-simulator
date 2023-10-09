
use crate::graphics::{GraphicsProgram, GraphicsContext};
use std::ffi::{CStr, CString};
use std::collections::HashMap;
use crate::util::print_type_of;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    dpi::LogicalSize
};

pub type WGPUGraphics = GraphicsContext<wgpu::Instance, winit::window::Window, winit::event_loop::EventLoop<()>>;
impl GraphicsProgram for WGPUGraphics {
    unsafe fn create_shader_program(vertex_shader_source: &str, frag_shader_source: &str) -> u32 {
        return 0;
    }
    fn swap_window(&self) {
        
    }
    fn get_backend_info(&self) {
        
    }
    fn default_state(&self) {
        
    }

    fn new (width: u32, height: u32) -> Self{
        let event = Box::new(EventLoop::new());
        let window = Window::new(&event).expect("unable to create winit window");
        window.set_inner_size(LogicalSize::new(width, height));
        let instance = Box::<wgpu::Instance>::default();
        async {
            let adapter = Box::new(
                instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some( 
                        & unsafe { instance.create_surface(&window) }.expect("unable to create surface")
                        )
                })
                .await
                .expect("unable to find appropriate adapter"));
            let (device, queue) = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
                )
                .await
                .expect("Failed to get device");
        };
        fn void_fn(p:&mut WGPUGraphics) {panic!("No callback function set!");}
        Self {
            attr_map: HashMap::new(),
            preloop_fn: void_fn,
            input_fn: void_fn,
            update_fn: void_fn,
            render_fn: void_fn,
            width, 
            height,
            window: Box::new(window),
            event,
            backend:instance,
            quit_loop:false,
            sdl_initialized: true,
            backend_initialized: true,
            red_channel_bg: 0.2,
            blue_channel_bg: 0.2,
            green_channel_bg: 0.2,
            alpha_channel_bg: 0.2,
        }
    }
}
