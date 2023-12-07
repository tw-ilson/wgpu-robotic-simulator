use crate::bindings::*;
use crate::graphics::Vertex;
use crate::texture;
use crate::wgpu_program::WGPUGraphics;
use anyhow::*;
use wgpu::core::binding_model;
use wgpu::hal::vulkan::ShaderModule;
use std::fs;
use std::path;
use std::str::FromStr;
use itertools::Itertools;

pub trait CompileShaders {
    // Produces compiled shader module from WGSL
    fn compile_wgsl(&mut self, name: &str, source: &str) -> wgpu::ShaderModule;
    // Produces compiled shader module from GLSL
    fn compile_glsl(
        &mut self,
        source: &str,
        stage: wgpu::naga::ShaderStage,
    ) -> wgpu::ShaderModule;
}

impl CompileShaders for WGPUGraphics<'_> {
    fn compile_wgsl(&mut self, name: &str, source: &str) -> wgpu::ShaderModule {
        let desc = wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            };
        self.device().create_shader_module(desc)
    }
    fn compile_glsl(
        &mut self,
        source: &str,
        stage: wgpu::naga::ShaderStage,
    ) -> wgpu::ShaderModule {
        let label = format!("{:?}", stage);
        let desc = wgpu::ShaderModuleDescriptor {
                label: Some(&label),
                source: wgpu::ShaderSource::Glsl {
                    shader: source.into(),
                    stage,
                    defines: naga::FastHashMap::default(),
                }
            };
        self.device().create_shader_module(desc)
    }
}

pub trait CreatePipeline {
    // Produces a RenderPipeline from shader source string
    fn create_render_pipeline(&mut self, shader_source: &str) -> Result<wgpu::RenderPipeline>;

    //Produces a ComputePipeline from shader source string
    fn create_compute_pipeline(&mut self, shader_source: &str, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> Result<wgpu::ComputePipeline>;
}
impl CreatePipeline for WGPUGraphics<'_> {
    fn create_compute_pipeline(&mut self, shader_source: &str, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> Result<wgpu::ComputePipeline> {
        let shader_module = self.compile_wgsl("compute shader", &shader_source);
        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        };

        let pipeline_layout = self
            .backend
            .device
            .create_pipeline_layout(&pipeline_layout_desc);

        let pipeline = self.backend.device
            .create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor{
                    label: Some("compute pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &shader_module,
                    entry_point: "main",
                });
        Ok(pipeline)
    }
    fn create_render_pipeline(&mut self, shader_source: &str) -> Result<wgpu::RenderPipeline> {
        let shader_module = self.compile_wgsl("vertex/fragment shader", &shader_source);

        let bind_group_layouts;
        let pipeline_layout_desc = 
            if self.backend.bindings.is_some() {
                bind_group_layouts = [
                    self.camera_bind_layout(),
                    self.light_bind_layout(),
                    self.transform_bind_layout(),
                ];
                wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                }
            } else {
              wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }
            };
        let pipeline_layout = self
            .backend
            .device
            .create_pipeline_layout(&pipeline_layout_desc);
        let pipeline = self.backend
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            });
        Ok(pipeline)
    }
}
