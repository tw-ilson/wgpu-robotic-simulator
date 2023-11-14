use crate::graphics::{Vertex};
use std::{convert::{From, Into}, hash::Hash};
use itertools::Itertools;
use std::ops::Range;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}

#[non_exhaustive]
pub enum MeshType {
    STL(String),
    OBJ(String),
}

#[derive(Debug, Clone)]
pub struct Mesh {
    faces: Vec<Triangle>
}
fn parse_stl(fstring:String) -> Mesh {
        let lines:Vec<&str> = {
            let mut lines = fstring.lines();
            match lines.next() {
                Some(s) => { if !s.trim().starts_with("solid") {panic!()}},
                _=> panic!(),
            }
            lines
        }.collect_vec();
        Mesh {
            faces: {
                let mut k = 0;
                lines.iter().filter_map(
                    |line| {
                            fn get_vert(s: &str) -> glm::Vec3{
                                let mut toks = s.split_whitespace().into_iter();
                                assert!(Some("vertex") == toks.next());
                                [
                                    toks.next().unwrap().parse::<f32>().unwrap(),
                                    toks.next().unwrap().parse::<f32>().unwrap(),
                                    toks.next().unwrap().parse::<f32>().unwrap()
                                ].into()
                            }
                            
                            let r = if line.split_whitespace().next().unwrap() == "facet" {
                                Some(
                                    Triangle {
                                        vertices: [
                                                get_vert(lines.clone()[k+2]).into(),
                                                get_vert(lines.clone()[k+3]).into(),
                                                get_vert(lines.clone()[k+4]).into(),
                                            ]
                                    }
                                )
                            } else {
                                None
                            };
                            k += 1;
                            r
                }).collect()
            }
        }
    }

impl From<MeshType> for Mesh {
    fn from(mesh_type: MeshType) -> Self {
        match mesh_type {
            MeshType::STL(fstring) => parse_stl(fstring),
            // OBJ(fstring) -> parse_obj(fstring),
            _=> panic!("type unsupported")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Polyhedron {
    pub verts: Vec<Vertex>,
    // edges: Vec<(u16,u16)>
    pub indices: Vec<u16>,
}

impl Polyhedron {
    pub fn from_fast(mesh:Mesh) -> Self {
        Self {
            verts: mesh.faces.iter().flat_map(|tri| tri.vertices).collect(),
            indices: (0..mesh.faces.len() as u16).flat_map(|fi|{let k:u16=fi*3;k..k+3}).collect(),
        }
    }
    pub fn calculate_normals(&mut self) {
        let mut i = 0;
        while i < self.indices.len() {
            let edge1 = self.verts[self.indices[i+1] as usize].position - self.verts[self.indices[i] as usize].position;
            let edge2 = self.verts[self.indices[i+2] as usize].position - self.verts[self.indices[i+1] as usize].position;
            let normal = glm::cross(&edge1, &edge2);
            self.verts[self.indices[i] as usize].normal = normal;
            self.verts[self.indices[i+1] as usize].normal = normal;
            self.verts[self.indices[i+2] as usize].normal = normal;
            i=i+3;
        }
    }
    pub fn scale(&mut self, factor: f32) {
        for mut v in &mut self.verts {
            v.position = v.position * factor;
        }
    }
}

impl From<Mesh> for Polyhedron {
    // create efficient index buffer -- adds overhead
    fn from(mesh: Mesh) -> Self {
        let verts: Vec<Vertex> = mesh.faces
            .iter()
            .flat_map(|tri| tri.vertices.iter().cloned())
            .dedup()
            .collect_vec();
        let indices: Vec<u16> = mesh.faces
            .iter()
            .flat_map(|tri| {
                let mut indices:Vec<u16> = Vec::new();
                for i in 0..3 {
                    let idx1 = verts.iter().position(|&v_b| v_b == tri.vertices[i]).unwrap();
                    indices.push(idx1 as u16);
                };
                indices
            }).collect();
       let mut poly = Self { verts, indices };
       poly.calculate_normals();
       poly
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Polyhedron,
        // material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Polyhedron,
        // material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

}

// impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
// where
//     'b: 'a,
// {
//     fn draw_mesh(
//         &mut self,
//         mesh: &'b Polyhedron,
//         // material: &'b Material,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.draw_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
//     }
//
//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &'b Polyhedron,
//         // material: &'b Material,
//         instances: Range<u32>,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         // self.set_bind_group(0, &material.bind_group, &[]);
//         self.set_bind_group(1, camera_bind_group, &[]);
//         self.set_bind_group(2, light_bind_group, &[]);
//         self.draw_indexed(0..mesh.num_elements, 0, instances);
//     }
// }
