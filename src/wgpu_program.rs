use crate::{
    camera::{Camera, CameraController, CameraUniform},
    light::{Light, LightUniform},
    graphics::{Color, ContextFlags, GraphicsContext, GraphicsProgram, Vertex},
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::collections::HashMap;
use itertools::izip;
use wgpu::{util::DeviceExt, Device, Surface};
use winit::{dpi::PhysicalSize, event_loop::{EventLoop, }, event::WindowEvent, window::{Window, WindowBuilder}};

pub struct WGPUState {
    size: winit::dpi::PhysicalSize<u32>,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
    // pipeline: Option<wgpu::RenderPipeline>,
    config: wgpu::SurfaceConfiguration,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    light: Light,
    light_uniform: LightUniform,
    bindings: Bindings, 
    num_vertices: u32,
    num_indices: u32,
}
// [camera, light] 
struct Bindings {
    names: Vec<String>,
    bind_layouts: Vec<wgpu::BindGroupLayout>,
    bind_groups: Vec<wgpu::BindGroup>,
}
impl Bindings {
    fn new() -> Self {
        Self { names: Vec::new(), bind_layouts: Vec::new(), bind_groups: Vec::new() }
    }
    fn new_bind_group_layout(&mut self, name: &str, device: &wgpu::Device) {
        self.names.push(name.to_string());
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some(&(name.to_string() + "_layout")),
            });
        self.bind_layouts.push(bind_group_layout);
    }
    pub fn create_bind_groups(&mut self, device: &wgpu::Device, buffers: &[&wgpu::Buffer]) {
        assert!(buffers.len() == self.bind_layouts.len());
        for (name, layout, buffer) in izip!(self.names.iter(), self.bind_layouts.iter(), buffers) {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some(name),
            });
            self.bind_groups.push(bind_group);
        }
    }
}

fn retrieve_adapter_device(
    instance: &wgpu::Instance,
    window: &Window,
) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
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
                    } else {
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
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[allow(dead_code)]
pub type WGPUGraphics = GraphicsContext<WGPUState, Window, wgpu::Buffer>;
impl WGPUGraphics {
    // convenience accessors for state
    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        return &self.backend.size;
    }
    pub fn instance(&self) -> &wgpu::Instance {
        &self.backend.instance
    }
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.backend.adapter
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.backend.device
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.backend.surface
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.backend.queue
    }
    // pub fn pipeline(&self) -> &wgpu::RenderPipeline {
    //     match &self.backend.render_pipeline {
    //         Some(pipeline) => pipeline,
    //         None => panic!("Accessed pipeline before creation!"),
    //     }
    // }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.backend.config
    }
    pub fn n_vert(&self) -> u32 {
        self.backend.num_vertices
    }
    pub fn n_indices(&self) -> u32 {
        self.backend.num_indices
    }
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.backend.camera 
    }
    pub fn camera_controller(&mut self) -> &mut CameraController {
        &mut self.backend.camera_controller
    }
    pub fn bind_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.backend.bindings.bind_layouts.iter().collect()
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.backend.bindings.bind_groups[0]
    }
    pub fn light_bind_group(&self) -> &wgpu::BindGroup {
        &self.backend.bindings.bind_groups[1]
    }
    pub fn create_bind_groups(&mut self, buffers: &[&wgpu::Buffer]) {
        let device = &self.backend.device;
        self.backend.bindings.create_bind_groups(device, buffers);
    }

    // helpers to create buffers
    fn create_buffer<T: Zeroable + Pod>(
        &mut self,
        name: &str,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        let buffer = self
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: cast_slice(data),
                usage,
            });
        // self.attr_map.insert(String::from(name), buffer);
        buffer
    }
    pub fn create_vertex_buffer(&mut self, vertices: Vec<Vertex>) -> wgpu::Buffer {
        self.backend.num_vertices = vertices.len() as u32;
        self.create_buffer(
            "Vertex Buffer",
            vertices.as_slice(),
            wgpu::BufferUsages::VERTEX,
        )
    }
    pub fn create_index_buffer(&mut self, indices: Vec<u16>) -> wgpu::Buffer {
        self.backend.num_indices = indices.len() as u32;
        self.create_buffer(
            "Index Buffer",
            indices.as_slice(),
            wgpu::BufferUsages::INDEX,
        )
    }
    pub fn update_camera(&mut self, camera_buffer: &wgpu::Buffer) {
        self.backend.camera_controller.update(&mut self.backend.camera);
        self.backend.camera.update_view_proj(&mut self.backend.camera_uniform);
        // self.backend.queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[self.backend.camera_uniform]));
        self.assign_uniform(camera_buffer, self.backend.camera_uniform);
    }
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool{
        self.backend.camera_controller.process_keyboard(event)
    }
    pub fn mouse_look(&mut self, mouse_x: f32, mouse_y: f32) {
        self.backend.camera_controller.mouse_look(
            &mut self.backend.camera, mouse_x, mouse_y)
    }
    pub fn create_camera_buffer(&mut self) -> wgpu::Buffer {
        self.backend.camera.update_view_proj(&mut self.backend.camera_uniform);
        self.create_buffer(
            "Camera Buffer",
            &[self.backend.camera_uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    }

    //Lights
    pub fn create_light_buffer(&mut self) -> wgpu::Buffer {
        self.create_buffer("Light VB" , &[self.backend.light.uniform], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }

    pub fn update_light(&mut self, light_buffer: &wgpu::Buffer) {
        self.backend.light_uniform.update();
        self.assign_uniform(light_buffer, self.backend.light_uniform);
    }

    pub fn assign_uniform<T: Zeroable + Pod>(&mut self, buffer: &wgpu::Buffer, data: T) {
        self.backend.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }

    //constructor
    pub fn new(width: u32, height: u32, event: &EventLoop<()>) -> Self {
        // let window = Window::new(event).expect("unable to create winit window");
        let window = WindowBuilder::new().build(event).expect("unable to create winit window");
        window.set_cursor_grab(winit::window::CursorGrabMode::Confined).unwrap();
        window.set_cursor_visible(false);
        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            // use winit::dpi::PhysicalSize;
            // program.window.set_inner_size(PhysicalSize::new(width, height));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let size = PhysicalSize::new(width, height);
        window.set_inner_size(size);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        let surface =
            unsafe { instance.create_surface(&window) }.expect("unable to create surface");
        let (adapter, device, queue) = retrieve_adapter_device(&instance, &window);

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(swapchain_capabilities.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: swapchain_capabilities.present_modes[0],
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        let camera = Camera::new(width, height);
        let camera_controller = CameraController::default();
        // let camera_buffer = program.create_camera_buffer();
        let light = Light::new(None);

        let camera_uniform = CameraUniform::new();
        let light_uniform = LightUniform::new();

        let mut bindings = Bindings::new();
        bindings.new_bind_group_layout("camera_bind_group", &device);
        bindings.new_bind_group_layout("light_bind_group", &device);

        let mut program = Self {
            attr_map: HashMap::new(),
            width,
            height,
            window,
            // event,
            backend: WGPUState {
                instance,
                surface,
                adapter,
                device,
                queue,
                size,
                config,
                camera,
                camera_controller,
                camera_uniform,
                light,
                light_uniform,
                bindings,
                num_vertices: 0,
                num_indices: 0,
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
            },
        };
        program.default_state();
        program
    }
}

impl GraphicsProgram for WGPUGraphics {
    fn swap_window(&self) {}
    fn get_backend_info(&self) {

    }
    fn default_state(&mut self) {
        self.backend.surface.configure(self.device(), self.config());
    }
}
