use crate::{
    bindings::*,
    camera::{Camera, CameraController, CameraUniform},
    geometry::{Polyhedron, Transform},
    graphics::{Color, ContextFlags, GraphicsContext, GraphicsProgram, Vertex},
    light::{Light, LightUniform},
    texture::Texture,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::HashMap;
// use rayon::prelude::*;
use winit::{
    dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, keyboard::KeyCode, window::{Window, WindowBuilder}
};

pub struct WGPUState<'a> {
    // Device Configuration state
    pub size: winit::dpi::PhysicalSize<u32>,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub surface: wgpu::Surface<'a>,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub depth_texture: Texture,

    // Runtime state
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub light: Light,
    // pub light_uniform: LightUniform,
    pub bindings: Option<Bindings>,
}

async fn retrieve_adapter_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> anyhow::Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
    // NOTE: this must stay async. On web (wasm32) the adapter/device
    // requests resolve through the browser event loop, so blocking on them
    // (e.g. futures::executor::block_on) hangs or panics. Callers drive this
    // with spawn_local on wasm and block_on on native.
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(
                surface
            ),
        })
        .await
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No WebGL2/GPU adapter found. This demo needs a browser with WebGL2 enabled                  (hardware acceleration turned on)."
            )
        })?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                // TEXTURE_BINDING_ARRAY is not universally available on web
                // (e.g. the WebGL2 fallback), so only require it on native.
                required_features: if cfg!(target_arch = "wasm32") {
                    wgpu::Features::empty()
                } else {
                    wgpu::Features::TEXTURE_BINDING_ARRAY
                },
                // Need to do the spatial transforms on
                // shader!
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default()
            },
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get GPU device: {e:?}"))?;
    Ok((adapter, device, queue))
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
pub type WGPUGraphics<'a> = GraphicsContext<WGPUState<'a>, &'a Window, wgpu::Buffer>;
impl<'a> WGPUGraphics<'a> {
    // convenience accessors for state
    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        return &self.state.size;
    }
    pub fn instance(&self) -> &wgpu::Instance {
        &self.state.instance
    }
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.state.adapter
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.state.device
    }
    pub fn device_mut(&mut self) -> &mut wgpu::Device {
        &mut self.state.device
    }
    pub fn surface(&self) -> &wgpu::Surface {
        &self.state.surface
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.state.queue
    }
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.state.config
    }
    pub fn camera(&mut self) -> &mut Camera {
        &mut self.state.camera
    }
    pub fn camera_controller(&mut self) -> &mut CameraController {
        &mut self.state.camera_controller
    }
    pub fn bindings(&mut self) -> &mut Bindings {
        self.state.bindings.as_mut().unwrap()
    }
    pub fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.state.bindings.as_ref().unwrap().camera_bind_group
    }
    pub fn light_bind_group(&self) -> &wgpu::BindGroup {
        &self.state.bindings.as_ref().unwrap().camera_bind_group
    }
    pub fn transform_bind_groups(&self) -> &Vec<wgpu::BindGroup> {
        // &self.bindings().transform_bind_groups
        &self
            .state
            .bindings
            .as_ref()
            .unwrap()
            .transform_bind_groups
    }
    pub fn camera_bind_layout(&self) -> &wgpu::BindGroupLayout {
        &self.state.bindings.as_ref().unwrap().camera_bind_layout
    }
    pub fn light_bind_layout(&self) -> &wgpu::BindGroupLayout {
        &self.state.bindings.as_ref().unwrap().light_bind_layout
    }
    pub fn transform_bind_layout(&self) -> &wgpu::BindGroupLayout {
        &self
            .state
            .bindings
            .as_ref()
            .unwrap()
            .transform_bind_layout
    }

    // helpers to create buffers
    pub fn create_buffer<T: Zeroable + Pod>(
        &mut self,
        name: &str,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        let buffer = self
            .state
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
    pub fn create_index_buffer(&mut self, indices: &Vec<u32>) -> wgpu::Buffer {
        // self.backend.num_indices = indices.len() as u32;
        self.create_buffer(
            "Index Buffer",
            indices.as_slice(),
            wgpu::BufferUsages::INDEX,
        )
    }
    pub fn assign_buffer<T: Zeroable + Pod>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        self.state
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }
    
    pub fn assign_texture<T: Zeroable + Pod>(&self, texture: Texture, data: &[T]) {
        self.state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.size.width),
                rows_per_image: Some(texture.size.height),
            },
            texture.size,
        );
    }
    // pub fn assign_texture_array<T: Zeroable + Pod>(&self, texture_array:

    // Camera
    pub fn create_camera_buffer(&mut self) -> wgpu::Buffer {
        self.state
            .camera
            .update_view_proj(&mut self.state.camera_uniform);
        self.create_buffer(
            "Camera Buffer",
            &[self.state.camera_uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    }
    pub fn update_camera(&mut self, camera_buffer: &wgpu::Buffer) {
        self.state
            .camera_controller
            .update(&mut self.state.camera);
        self.state
            .camera
            .update_view_proj(&mut self.state.camera_uniform);
        self.assign_buffer(camera_buffer, &[self.state.camera_uniform]);
    }
    pub fn process_keyboard(&mut self, event: &KeyCode) {
        self.state.camera_controller.process_keyboard(event)
    }
    // pub fn mouse_look(&mut self, mouse_x: f32, mouse_y: f32) {
    //     self.backend.camera_controller.mouse_look(
    //         &mut self.backend.camera, mouse_x, mouse_y)
    // }

    //Lights
    pub fn create_light_buffer(&mut self) -> wgpu::Buffer {
        self.state
            .light
            .uniform
            .set(self.state.camera.get_eye_posn());
        self.create_buffer(
            "Light Buffer",
            &[self.state.light.uniform],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        )
    }

    pub fn update_light(&mut self, light_buffer: &wgpu::Buffer) {
        self.state
            .light
            .uniform
            .set(self.state.camera.get_eye_posn());
        self.assign_buffer(light_buffer, &[self.state.light.uniform]);
    }

    // Transforms
    pub fn update_transforms<T>(&mut self, t_buffers: &Vec<wgpu::Buffer>, t_data: T)
    where
        T: IntoIterator,
        T::Item: Borrow<Transform>,
    {
        std::iter::zip(t_buffers, t_data).for_each(|(b, t)| self.assign_buffer(b, &[*t.borrow()]))
    }

    pub fn create_transform_buffers<T>(&mut self, t_list: T) -> Vec<wgpu::Buffer>
    where
        T: IntoIterator,
        T::Item: Borrow<Transform>,
    {
        t_list
            .into_iter()
            .map(|t| {
                self.create_buffer(
                    "Transform Buffer",
                    &[*t.borrow()],
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                )
            })
            .collect()
    }

    // Mesh
    pub fn create_mesh_buffer(&mut self, poly: &Polyhedron) -> MeshBuffer {
        MeshBuffer {
            n_indices: poly.indices().len() as u32,
            vertex_buffer: self.create_vertex_buffer(&poly.verts),
            index_buffer: self.create_index_buffer(&poly.indices),
        }
    }
    pub fn create_mesh_buffers<T>(&mut self, mesh_list: T) -> Vec<MeshBuffer>
        where 
            T: IntoIterator,
            T::Item: Borrow<Polyhedron>,
        {
            mesh_list
                .into_iter()
                .map(|poly|
                     self.create_mesh_buffer(poly.borrow())
                     )
                .collect()
        }

    pub fn assign_mesh_buffer(&mut self, poly: &Polyhedron, buffer: &MeshBuffer) {
        self.assign_buffer(&buffer.index_buffer, poly.indices());
        self.assign_buffer(&buffer.vertex_buffer, poly.verts());
    }

    //constructor
    pub async fn new(width: u32, height: u32, window: &'a Window) -> anyhow::Result<Self> {
        // let window = Window::new(event).expect("unable to create winit window");
        // Pointer lock is desktop-only: on web requestPointerLock requires a
        // user gesture and doesn't exist at all on iOS Safari, where calling
        // it throws and then panics inside winit. The examples don't use
        // mouse-look anyway, so skip cursor capture entirely on web.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .is_err()
            {}
            window.set_cursor_visible(false);
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(
                        window.canvas().expect("winit window has no canvas"),
                    );
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let size = PhysicalSize::new(width, height);
        let _ = window.request_inner_size(size);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        let surface = instance.create_surface(wgpu::SurfaceTarget::from(window)).expect("unable to create surface");

        let (adapter, device, queue) = retrieve_adapter_device(&instance, &surface).await?;

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
            desired_maximum_frame_latency: 2,
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera = Camera::new(width, height);
        let camera_controller = CameraController::default();
        let camera_uniform = CameraUniform::new();
        let light = Light::new(None);

        let mut program = Self {
            attr_map: HashMap::new(),
            width,
            height,
            window,
            // event,
            state: WGPUState {
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
                depth_texture,
                bindings: None,
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
        Ok(program)
    }

    pub fn draw_mesh_list(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        buffer_list: &Vec<MeshBuffer>,
    ) {
        // self.set_clear_color((1.0, 1.0, 1.0, 1.0));
        let output = self
            .state
            .surface
            .get_current_texture()
            .expect("failed to get current texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.bg_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(pipeline);

            render_pass.draw_mesh_list(
                &buffer_list,
                self.camera_bind_group(),
                self.light_bind_group(),
                self.transform_bind_groups(),
            )
        }
        self.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }
    pub fn create_bindings(
        &mut self,
        light_buffer: &wgpu::Buffer,
        camera_buffer: &wgpu::Buffer,
        transform_buffers: &Vec<wgpu::Buffer>,
    ) {
        use crate::bindings::*;
        let camera_bind_layout = new_uniform_bind_group_layout(
            &self.state.device,
            "camera_bind_layout",
            &[uniform_layout_entry()],
        );
        let light_bind_layout = new_uniform_bind_group_layout(
            &self.state.device,
            "light_bind_layout",
            &[uniform_layout_entry()],
        );
        let transform_bind_layout = new_uniform_bind_group_layout(
            &self.state.device,
            "transform_bind_layout",
            &[uniform_layout_entry()],
        );
        let camera_bind_group = create_uniform_bind_group(
            &self.state.device,
            &camera_bind_layout,
            camera_buffer,
            "camera_bind_group",
        );
        let light_bind_group = create_uniform_bind_group(
            &self.state.device,
            &light_bind_layout,
            light_buffer,
            "light_bind_group",
        );
        let transform_bind_groups = transform_buffers
            .iter()
            .enumerate()
            .map(|(i, buffer)| {
                create_uniform_bind_group(
                    &self.state.device,
                    &transform_bind_layout,
                    buffer,
                    &format!("transform_bind_group_{}", i),
                )
            })
            .collect();
        self.state.bindings = Some(Bindings {
            camera_bind_layout,
            light_bind_layout,
            camera_bind_group,
            light_bind_group,
            transform_bind_layout,
            transform_bind_groups,
        });
    }
}

impl GraphicsProgram for WGPUGraphics<'_> {
    fn swap_window(&self) {}
    fn get_backend_info(&self) {
        println!("Device features:\n{:#?}", self.device().features());
        println!("Adapter info: \n{:#?}", self.adapter().get_info());
        // println!("Adapter features:\n{:#?}", self.adapter().features());
    }
    fn default_state(&mut self) {
        self.state
            .surface
            .configure(&self.state.device, self.config());
    }
}
impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {r:self.r, g:self.g, b:self.b, a:self.a}
    }
}

pub struct MeshBuffer {
    pub n_indices: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

pub trait DrawMeshBuffer<'a> {
    fn draw_mesh(
        &mut self,
        vao: &'a MeshBuffer,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        transform_bind_group: &'a [wgpu::BindGroup],
        transform_index: usize,
    );
    fn draw_mesh_list(
        &mut self,
        vao_list: &'a Vec<MeshBuffer>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        transform_bind_group: &'a [wgpu::BindGroup],
    );
}

impl<'a, 'b> DrawMeshBuffer<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        vao: &'b MeshBuffer,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        transform_bind_groups: &'a [wgpu::BindGroup],
        transform_index: usize,
    ) {
        self.set_bind_group(0, &camera_bind_group, &[]);
        self.set_bind_group(1, &light_bind_group, &[]);
        self.set_bind_group(2, &transform_bind_groups[transform_index], &[]);
        self.set_vertex_buffer(0, vao.vertex_buffer.slice(..));
        self.set_index_buffer(vao.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..vao.n_indices, 0, 0..1);
    }
    fn draw_mesh_list(
        &mut self,
        vao_list: &'a Vec<MeshBuffer>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        transform_bind_groups: &'a [wgpu::BindGroup],
    ) {
        for (i, buffer) in vao_list.iter().enumerate() {
            self.draw_mesh(
                buffer,
                camera_bind_group,
                light_bind_group,
                transform_bind_groups,
                i,
            );
        }
    }
}
