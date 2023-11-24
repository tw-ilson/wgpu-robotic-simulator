use crate::{
    texture::Texture,
    bindings::*,
    camera::{Camera, CameraController, CameraUniform},
    light::{Light, LightUniform},
    graphics::{Color, ContextFlags, GraphicsContext, GraphicsProgram, Vertex}, 
    geometry::{Transform, Polyhedron, MeshBuffer, DrawMeshBuffer},
};
use itertools::Itertools;
use bytemuck::{cast_slice, Pod, Zeroable};
use std::collections::HashMap;
// use rayon::prelude::*;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, event::WindowEvent, window::{Window, WindowBuilder}};

pub struct WGPUState {
    // Device Configuration state
    pub size: winit::dpi::PhysicalSize<u32>,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub depth_texture: Texture,

    // Runtime state
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub light: Light,
    pub light_uniform: LightUniform,
    pub bindings: Bindings, 
    // num_vertices: u32,
    // num_indices: u32,
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
                    features: wgpu::Features::empty(), //| wgpu::Features::BUFFER_BINDING_ARRAY, //
                                                       // Need to do the spatial transforms on
                                                       // shader!
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
    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.backend.device
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.backend.surface
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.backend.queue
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.backend.config
    }
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.backend.camera 
    }
    pub fn camera_controller(&mut self) -> &mut CameraController {
        &mut self.backend.camera_controller
    }
    pub fn bindings(&mut self) -> &mut Bindings {
        &mut self.backend.bindings
    }
    pub fn bind_groups(&self) -> Vec<&wgpu::BindGroup> {
        self.backend.bindings.bind_groups.iter().collect()
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.backend.bindings.bind_groups[0]
    }
    pub fn light_bind_group(&self) -> &wgpu::BindGroup {
        &self.backend.bindings.bind_groups[1]
    }
    pub fn transform_bind_group(&self) -> &wgpu::BindGroup {
        &self.backend.bindings.bind_groups[2]
    }

    // helpers to create buffers
    pub fn create_buffer<T: Zeroable + Pod>(
        &mut self,
        name: &str,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        let buffer = self.backend
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: cast_slice(data),
                usage,
            });
        // self.attr_map.insert(String::from(name), buffer);
        buffer
    }
    pub fn create_vertex_buffer(&mut self, vertices: &Vec<Vertex>) -> wgpu::Buffer {
        // self.backend.num_vertices = vertices.len() as u32;
        self.create_buffer(
            "Vertex Buffer",
            vertices.as_slice(),
            wgpu::BufferUsages::VERTEX,
        )
    }
    pub fn create_index_buffer(&mut self, indices: &Vec<u16>) -> wgpu::Buffer {
        // self.backend.num_indices = indices.len() as u32;
        self.create_buffer(
            "Index Buffer",
            indices.as_slice(),
            wgpu::BufferUsages::INDEX,
        )
    }
    pub fn assign_uniform<T: Zeroable + Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.backend.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }

    // Camera
    pub fn create_camera_buffer(&mut self) -> wgpu::Buffer {
        self.backend.camera.update_view_proj(&mut self.backend.camera_uniform);
        self.create_buffer(
            "Camera Buffer",
            &[self.backend.camera_uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    }
    pub fn update_camera(&mut self, camera_buffer: &wgpu::Buffer) {
        self.backend.camera_controller.update(&mut self.backend.camera);
        self.backend.camera.update_view_proj(&mut self.backend.camera_uniform);
    }
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool{
        self.backend.camera_controller.process_keyboard(event)
    }
    pub fn mouse_look(&mut self, mouse_x: f32, mouse_y: f32) {
        self.backend.camera_controller.mouse_look(
            &mut self.backend.camera, mouse_x, mouse_y)
    }
    

    //Lights
    pub fn create_light_buffer(&mut self) -> wgpu::Buffer {
        self.create_buffer(
            "Light Buffer",
            &[self.backend.light.uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }

    pub fn update_light(&mut self, light_buffer: &wgpu::Buffer) {
        // self.backend.light_uniform.update();
        self.backend.light_uniform.set(self.backend.camera.get_eye_posn());
    }

    // Mesh
    pub fn create_transform_buffer(&mut self, mesh_list: &Vec<Polyhedron>) -> wgpu::Buffer {
        self.create_buffer(
            "Transform Buffer",
            // mesh_list.iter().map(|mesh| mesh.transform).collect_vec().as_slice(),
            &[Transform::default()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }

    pub fn create_mesh_buffer(&mut self, poly: &Polyhedron) -> MeshBuffer {
        MeshBuffer {
            n_indices: poly.indices().len() as u32,
            vertex_buffer: self.create_vertex_buffer(&poly.verts),
            index_buffer: self.create_index_buffer(&poly.indices),
        }
    }

    pub fn draw_mesh_list(&mut self, pipeline: &wgpu::RenderPipeline, buffer_list: &Vec<MeshBuffer>, mesh_list: &Vec<Polyhedron>, camera_buffer: &wgpu::Buffer, light_buffer: &wgpu::Buffer, transform_buffer: &wgpu::Buffer) {
        // self.set_clear_color((1.0, 1.0, 1.0, 1.0));
        let output = self
            .backend.surface
            .get_current_texture()
            .expect("failed to get current texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.backend.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.backend.depth_texture.view,
                        depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            render_pass.set_pipeline(pipeline);
            self.assign_uniform(camera_buffer, &[self.backend.camera_uniform]);
            self.assign_uniform(light_buffer, &[self.backend.light_uniform]);
            // self.assign_uniform(transform_buffer, mesh_list.iter().map(|mesh|mesh.transform).collect_vec().as_slice());

            render_pass.draw_mesh_list(&buffer_list, self.camera_bind_group(), self.light_bind_group(), self.transform_bind_group())
        }
        self.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }

    //constructor
    pub fn new(width: u32, height: u32, event: &EventLoop<()>) -> Self {
        // let window = Window::new(event).expect("unable to create winit window");
        let window = WindowBuilder::new().build(event).expect("unable to create winit window");
        if window.set_cursor_grab(winit::window::CursorGrabMode::Locked).is_err() {

        }
        window.set_cursor_visible(false);

        // #[cfg(target_arch = "wasm32")]
        // {
        //     // Winit prevents sizing with CSS, so we have to set
        //     // the size manually when on web.
        //     // use winit::dpi::PhysicalSize;
        //     // program.window.set_inner_size(PhysicalSize::new(width, height));
        //
        //     use winit::platform::web::WindowExtWebSys;
        //     web_sys::window()
        //         .and_then(|win| win.document())
        //         .and_then(|doc| {
        //             let dst = doc.get_element_by_id("wasm-example")?;
        //             let canvas = web_sys::Element::from(window.canvas());
        //             dst.append_child(&canvas).ok()?;
        //             Some(())
        //         })
        //         .expect("Couldn't append canvas to document body.");
        // }

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

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera = Camera::new(width, height);
        let camera_controller = CameraController::default();
        let camera_uniform = CameraUniform::new();
        let light = Light::new(None);

        let light_uniform = LightUniform::new();

        let bindings = Bindings::new();

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
                depth_texture,
                bindings,
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
        println!("Device features:\n{:#?}", self.device().features());
        println!("Adapter info: \n{:#?}", self.adapter().get_info());
        println!("Adapter features:\n{:#?}", self.adapter().features());
    }
    fn default_state(&mut self) {
        self.backend.surface.configure(&self.backend.device, self.config());
    }
}
