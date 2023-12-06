use crate::bindings::*;
use crate::graphics::Vertex;
use crate::texture;
use crate::wgpu_program::WGPUGraphics;
use itertools::Itertools;

pub trait CompilePipeline {
    fn compile_wgsl(&mut self, name: &str, source: &str) -> wgpu::ShaderModule;
    fn compile_glsl(&mut self,source: &str,stage: Option<wgpu::naga::ShaderStage>,
    ) -> wgpu::ShaderModule;
    fn create_shader_program(&mut self, shader_source: &str) -> wgpu::RenderPipeline;
    fn create_render_pipeline(
        &mut self,
        pipeline_layout: wgpu::PipelineLayout,
        shader_module: wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline;
}

impl CompilePipeline for WGPUGraphics<'_> {
    fn compile_wgsl(&mut self, name: &str, source: &str) -> wgpu::ShaderModule {
        self.device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            })
    }
    fn compile_glsl(
        &mut self,
        source: &str,
        stage: Option<wgpu::naga::ShaderStage>,
    ) -> wgpu::ShaderModule {
        self.device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("{:?}", stage)),
                source: wgpu::ShaderSource::Glsl {
                    shader: source.into(),
                    stage: stage.expect("Shader type must be defined for GLSL"),
                    defines: naga::FastHashMap::default(),
                },
            })
    }

    fn create_shader_program(
        &mut self,
        shader_source: &str,
    ) -> wgpu::RenderPipeline {
        let shader_module = self.compile_wgsl("vertex/fragment shader", shader_source);
        let bind_group_layouts = [
            self.camera_bind_layout(),
            self.light_bind_layout(),
            self.transform_bind_layout()];
        let pipeline_layout_desc = &wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        };
        let pipeline_layout = self.backend.device.create_pipeline_layout(&pipeline_layout_desc);
        self.create_render_pipeline(pipeline_layout, shader_module)
    }

    fn create_render_pipeline(
        &mut self,
        pipeline_layout: wgpu::PipelineLayout,
        shader_module: wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        self.backend.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                        alpha: wgpu::BlendComponent::REPLACE,
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
                conservative: false,
            },
            // depth_stencil: depth_format,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
