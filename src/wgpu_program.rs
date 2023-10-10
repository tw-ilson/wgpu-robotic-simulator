use crate::graphics::{GraphicsContext, GraphicsProgram};
use crate::util::print_type_of;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use wgpu::{Adapter, Device, Instance};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn retrieve_adapter_device(instance: &wgpu::Instance, window: &Window) -> wgpu::Device {
    let device_fut = async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(
                    &unsafe { instance.create_surface(&window) }.expect("unable to create surface"),
                ),
            })
            .await
            .expect("unable to find appropriate adapter");
        let (device, queue) = adapter
            .request_device(
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
            .expect("F ailed to get device");

        device
    };
    return futures::executor::block_on(device_fut);
}

#[non_exhaustive]
enum WGPUResource {
    Pipeline(wgpu::RenderPipeline),
}

pub type WGPUGraphics = GraphicsContext<Instance, Window, EventLoop<()>, WGPUResource>;
impl GraphicsProgram for WGPUGraphics {
    unsafe fn create_shader_program(
        &mut self,
        vertex_shader_source: &str,
        frag_shader_source: &str,
    ) {
        use naga::ShaderStage::{Fragment, Vertex};
        let device = retrieve_adapter_device(&self.backend, &self.window);
        fn compile_shader(source: &str, device: &Device, shader_type: naga::ShaderStage) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{:?}", shader_type)),
                source: wgpu::ShaderSource::Glsl {
                    shader: Cow::Borrowed(source),
                    stage: shader_type,
                    defines: naga::FastHashMap::default(),
                },
            })
        }
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &compile_shader(vertex_shader_source, &device, Vertex),
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &compile_shader(frag_shader_source, &device, Fragment),
                entry_point: "fs_main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        self.attr_map.insert("PIPELINE".to_string(), WGPUResource::Pipeline(render_pipeline));
    }
    fn swap_window(&self) {}
    fn get_backend_info(&self) {}
    fn default_state(&self) {}
    fn new(width: u32, height: u32) -> Self {
        let event = Box::new(EventLoop::new());
        let window = Window::new(&event).expect("unable to create winit window");
        window.set_inner_size(LogicalSize::new(width, height));
        let instance = Box::<wgpu::Instance>::default();
        fn void_fn(_p: &mut WGPUGraphics) {
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
            window: Box::new(window),
            event,
            backend: instance,
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
