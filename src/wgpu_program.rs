use crate::graphics::{Vertex, vertex_bytes, GraphicsContext, GraphicsProgram, ContextFlags, Color};
use crate::util::print_type_of;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use wgpu::{Adapter, Device, Instance, Surface, util::DeviceExt};
use winit::event::{KeyboardInput, ElementState, VirtualKeyCode};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn retrieve_adapter_device(instance: &wgpu::Instance, window: &Window, surface: &Surface) 
-> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
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
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
                    }
                    else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .expect("Failed to get device");
        (adapter, device, queue)
    };
    futures::executor::block_on(device_fut)
}
impl Vertex {
    // needs to be changed if Vertex is changed.
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[non_exhaustive]
// pub enum WGPUResource {
//     Pipeline(wgpu::RenderPipeline),
// }

pub struct WGPUState {
    size: winit::dpi::PhysicalSize<u32>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    num_vertices: Option<u32>
}

pub type WGPUGraphics = GraphicsContext<WGPUState, Window, wgpu::Buffer>;
impl WGPUGraphics {
    pub unsafe fn create_shader_program(
        &mut self,
        shader_source: &str,
    ) {
        fn compile_shader(source: &str, device: &Device, shader_type: Option<naga::ShaderStage>) -> wgpu::ShaderModule {
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
            // device.create_shader_module(wgpu::ShaderModuleDescriptor {
            //     label: Some(&format!("{:?}", shader_type)),
            //     source: wgpu::ShaderSource::Glsl {
            //         shader: Cow::Borrowed(source),
            //         stage: shader_type.expect("Shader type must be defined for GLSL"),
            //         defines: naga::FastHashMap::default(),
            //     },
            // })
        }
        let shader_module = compile_shader(shader_source, self.device(), None);
        let swapchain_capabilities = self.backend.surface.get_capabilities(&self.backend.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let pipeline_layout = self.backend.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = self.backend.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module, 
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config().format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires
                // Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative:false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled:false,
            },
            multiview: None,
        });
        self.backend.render_pipeline = Some(render_pipeline);
         // self.attr_map.insert(String::from("PIPELINE"), WGPUResource::Pipeline(render_pipeline));
    }
    fn create_buffer(&self, name: &str, data: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(name),
            contents: data,
            usage,
        })
    }
    pub fn create_vertex_buffer(&mut self, vertices: Vec<Vertex>) -> wgpu::Buffer {
        self.backend.num_vertices = Some(vertices.len() as u32);
        self.create_buffer("Vertex Buffer", vertex_bytes(&vertices), wgpu::BufferUsages::VERTEX)
    }
    // convenience accessors for state
    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> { return &self.backend.size }
    pub fn instance(&self) -> &wgpu::Instance { &self.backend.instance }
    pub fn adapter(&self) -> &wgpu::Adapter { &self.backend.adapter }
    pub fn device(&self) -> &wgpu::Device { &self.backend.device }
    pub fn surface(&self) -> &wgpu::Surface { &self.backend.surface }
    pub fn queue(&self) -> &wgpu::Queue { &self.backend.queue }
    pub fn pipeline(&self) -> &wgpu::RenderPipeline { 
        if let Some(pipeline) = &self.backend.render_pipeline {pipeline} else {panic!("Accessed pipeline before creation!")}
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration { 
        if let Some(config) = &self.backend.config {config}
        else {panic!("Accessed configuration before adapter configured")} 
    }
    pub fn n_vert(&self) -> u32 {self.backend.num_vertices.unwrap_or(0)}

    //constructor
    pub fn new(width: u32, height: u32, event: &EventLoop<()>) -> Self {
        let window = Window::new(event).expect("unable to create winit window");

        let size = PhysicalSize::new(width, height);
        window.set_inner_size(size);
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.expect("unable to create surface");
        let (adapter, device, queue) = retrieve_adapter_device(&instance, &window, &surface); 
        fn void_fn(_p: &mut WGPUGraphics) { panic!("No callback function set!"); }
        let mut program = Self {
            attr_map: HashMap::new(),
            width,
            height,
            window,
            // event,
            backend: WGPUState{ 
                instance,
                surface,
                adapter,
                device,
                queue,
                size, 
                config:None,
                render_pipeline:None,
                num_vertices:None
            },
            flags: ContextFlags {
                quit_loop: false,
                sdl_initialized: true,
                backend_initialized: true,
            },
            bg_color: Color {
                r: 0.2,
                b: 0.2,
                g: 0.2,
                a: 0.2,
            }
        };
        program.default_state();
        program
    }
}
impl GraphicsProgram for WGPUGraphics {
    fn swap_window(&self) {
    }
    fn get_backend_info(&self) {
    }
    fn default_state(&mut self) {
        // let surface = self.surface();
        let size = self.window.inner_size();
        let swapchain_capabilities = self.backend.surface.get_capabilities(&self.backend.adapter);
        let swapchain_format = swapchain_capabilities.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(swapchain_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        self.backend.config = Some(config);
        self.backend.surface.configure(self.device(), self.config());
    }
}
