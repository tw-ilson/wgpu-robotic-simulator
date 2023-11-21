use crate::graphics::Vertex;
use crate::texture;
use crate::wgpu_program::WGPUGraphics;
fn compile_wgsl(name: &str, source: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(name),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}
fn compile_glsl(
    source: &str,
    device: &wgpu::Device,
    stage: Option<wgpu::naga::ShaderStage>,
) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("{:?}", stage)),
        source: wgpu::ShaderSource::Glsl {
            shader: source.into(),
            stage: stage.expect("Shader type must be defined for GLSL"),
            defines: naga::FastHashMap::default(),
        },
    })
}

pub unsafe fn create_shader_program(
    program: &WGPUGraphics,
    shader_source: &str,
) -> wgpu::RenderPipeline {
    let shader_module = compile_wgsl("vertex/fragment shader", shader_source, program.device());
    let pipeline_layout =
        program
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &program.bind_layouts().as_slice(),
                push_constant_ranges: &[],
            });
    create_render_pipeline(program, pipeline_layout, shader_module)
}

fn create_render_pipeline(
    program: &WGPUGraphics,
    pipeline_layout: wgpu::PipelineLayout,
    shader_module: wgpu::ShaderModule,
    // depth_format: Option<wgpu::TextureFormat>,
) -> wgpu::RenderPipeline {
    program
        .device()
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
                    format: program.config().format,
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

fn light_shader_program(program: &WGPUGraphics, shader_source: &str) -> wgpu::RenderPipeline {
    let layout = program
        .device()
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Pipeline Layout"),
            bind_group_layouts: &program.bind_layouts().as_slice(),
            push_constant_ranges: &[],
        });
    create_render_pipeline(
        program,
        layout,
        compile_wgsl("Light Shader", shader_source, program.device()),
    )
}
