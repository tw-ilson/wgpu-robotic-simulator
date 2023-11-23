use std::num::NonZeroU32;

use itertools::izip;

use crate::wgpu_program::WGPUGraphics;

pub struct Bindings {
    pub names: Vec<String>,
    pub bind_layouts: Vec<wgpu::BindGroupLayout>,
    pub bind_groups: Vec<wgpu::BindGroup>,
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
        count: if count > 0 {NonZeroU32::new(count as u32)} else {None},
    }
}
pub trait ManageBindings {
    fn new_bind_group_layout(&mut self, name:&str, entries: &[wgpu::BindGroupLayoutEntry]);
    fn create_bind_groups(&mut self, buffers: &[&wgpu::Buffer]);

}
impl Bindings {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            bind_layouts: Vec::new(),
            bind_groups: Vec::new(),
        }
    }
}
impl ManageBindings for WGPUGraphics {
    fn new_bind_group_layout(
        &mut self,
        name: &str,
        entries: &[wgpu::BindGroupLayoutEntry],
    ) {
        self.backend.bindings.names.push(name.to_string());
        self.backend.bindings.bind_layouts.push(
            self.backend.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries,
                label: Some(&(name.to_string() + "_layout")),
            }),
        );
    }

    fn create_bind_groups(&mut self, buffers: &[&wgpu::Buffer]) {
        assert!(buffers.len() == self.bindings().bind_layouts.len());
        for (name, layout, buffer) in izip!(self.backend.bindings.names.iter(), self.backend.bindings.bind_layouts.iter(), buffers) {
            let bind_group = self.backend.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some(name),
            });
            self.backend.bindings.bind_groups.push(bind_group);
        }
    }
}
