use crate::{
    camera::{Camera, CameraController, CameraUniform},
    light::LightUniform,
    graphics::{Color, ContextFlags, GraphicsContext, GraphicsProgram, Vertex},
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::collections::HashMap;
use std::slice;
use wgpu::{util::DeviceExt, Device, Surface};
use winit::{dpi::PhysicalSize, event_loop::{EventLoop, }, event::WindowEvent, window::{Window, WindowBuilder}};

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
// [camera, light] 
struct Uniforms {
    bind_groups: Vec<wgpu::BindGroup>,
    bind_layouts: Vec<wgpu::BindGroupLayout>,
}
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
    uniforms: Uniforms, 
    num_vertices: u32,
    num_indices: u32,
}

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
        self.backend.uniforms.bind_layouts.iter().collect()
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        // &self.backend.camera_bind_group.as_ref().expect("camera uniform not yet assigned!")
        &self.backend.uniforms.bind_groups[0]
    }
    // pub unsafe fn create_shader_program(&mut self, shader_source: &str) {
    //     self.backend.render_pipeline = Some(render_pipeline);
    // }

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
    pub fn initialize_camera(&mut self) -> CameraUniform {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.backend.camera);
        camera_uniform
    }
    pub fn update_camera(&mut self) {
        self.backend.camera_controller.update(&mut self.backend.camera);
    }
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool{
        self.backend.camera_controller.process_keyboard(event)
    }
    pub fn mouse_look(&mut self, mouse_x: f32, mouse_y: f32) {
        self.backend.camera_controller.mouse_look(
            &mut self.backend.camera, mouse_x, mouse_y)
    }
    pub fn create_camera_buffer(&mut self, camera_uniform: &mut CameraUniform) -> wgpu::Buffer {
        camera_uniform.update_view_proj(&self.backend.camera);
        let camera_buffer = self.create_buffer(
            "Camera Buffer",
            &[*camera_uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let camera_bind_group = self.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.backend.uniforms.bind_layouts[0],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        self.backend.uniforms.bind_groups.push(camera_bind_group);
        camera_buffer
    }

    //Lights
    pub fn initialize_lights(&mut self) -> LightUniform {
        LightUniform::new()
    }

    pub fn create_light_buffer(&mut self, light_uniform: &mut LightUniform) -> wgpu::Buffer {
        let light_buffer = self.create_buffer("Light VB" , &[*light_uniform], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let light_bind_group = self.backend.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.backend.uniforms.bind_layouts[1],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        light_buffer
    }


    pub fn new_bind_group(&mut self, name: &str) {

        // let light_bind_group = self.backend.device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &self.backend.uniforms.bind_layouts[1],
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: light_buffer.as_entire_binding(),
        //     }],
        //     label: None,
        // });
    }

    //constructor
    pub fn new(width: u32, height: u32, event: &EventLoop<()>) -> Self {
        // let window = Window::new(event).expect("unable to create winit window");
        let window = WindowBuilder::new().build(event).expect("unable to create winit window");
        window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
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
        surface.configure(&device, &config);

        // can abstract this stuff better
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    label: Some("camera_bind_group_layout"),
                });
        let light_bind_group_layout = 
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("light_bind_group_layout"),
        });

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
                camera: Camera::new(width, height),
                camera_controller: CameraController::default(),
                uniforms: Uniforms {
                    bind_layouts: vec![camera_bind_group_layout,],
                    bind_groups: Vec::new(),
                },
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
