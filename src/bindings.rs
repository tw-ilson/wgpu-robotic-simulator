use std::num::NonZeroU32;
// use itertools::{izip, Itertools};
use std::borrow::Borrow;
use crate::texture::Texture;
use crate::wgpu_program::WGPUGraphics;

pub struct Bindings {
    // pub names: Vec<String>,
    pub camera_bind_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub light_bind_layout: wgpu::BindGroupLayout,
    pub light_bind_group: wgpu::BindGroup,
    pub transform_bind_layout: wgpu::BindGroupLayout,
    pub transform_bind_groups: Vec<wgpu::BindGroup>,
}

pub fn uniform_layout_entry() -> wgpu::BindGroupLayoutEntry {
    uniform_array_layout_entry(0, false)
}
pub fn uniform_array_layout_entry(
    count: usize,
    has_dynamic_offset: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset,
            min_binding_size: None,
        },
        count: if count > 0 {
            NonZeroU32::new(count as u32)
        } else {
            None
        },
    }
}
pub fn new_uniform_bind_group_layout(
    device: &wgpu::Device,
    name: &str,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries,
        label: Some(&(name.to_string() + "_layout")),
    })
}
pub fn create_uniform_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    name: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some(name),
    })
}
fn create_texture_bind_group_layout(
    device: &wgpu::Device
    ) -> wgpu::BindGroupLayout {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            })
}
fn create_texture_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &Texture,
) -> wgpu::BindGroup {
        device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: None,
        })
}
