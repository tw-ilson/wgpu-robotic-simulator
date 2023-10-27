use crate::graphics::{Vertex};
use std::{convert::{From, Into}, hash::Hash};
use itertools::Itertools;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}
#[derive(Debug, Clone)]
pub struct Mesh {
    faces: Vec<Triangle>
}
impl From<String> for Mesh {
    fn from(fstring: String) -> Self {
        let lines:Vec<&str> = {
            let mut lines = fstring.lines();
            match lines.next() {
                Some(s) => { if !s.trim().starts_with("solid") {panic!()}},
                _=> panic!(),
            }
            lines
        }.collect_vec();
        Self {
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
}

#[derive(Debug, Clone)]
pub struct Polyhedron {
    pub verts: Vec<Vertex>,
    // edges: Vec<(u16,u16)>
    pub indices: Vec<u16>,
}
impl Polyhedron {
    // fn edges(&self) -> &Vec<Vertex> {
    //     &self.verts
    // }
    fn check() {
        unimplemented!()
    }
}
impl From<Mesh> for Polyhedron {
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
                    // let idx2 = verts.iter().position(|&v_b| v_b == tri.vertices[(i+1)%3]).unwrap();
                    // edges.push(if idx1 < idx2 {
                    //     (idx1 as u16, idx2 as u16)
                    // } else {
                    //     (idx2 as u16, idx1 as u16)
                    // });
                };
                indices
            }).collect();
        // let edges: Vec<(u16, u16)> = edge_set.into_iter().collect();
       // get the correct set of edges
       Self { verts, indices }
    }
}
// impl Into<Vec<Triangle>> for Polyhedron {
//     fn into(self) -> Vec<Triangle> {
//         unimplemented!()
//     }
// }
//
