use crate::graphics::{Vertex};
use std::convert::{From, Into};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Rc<Vertex>; 3],
}

#[derive(Debug, Clone)]
struct Polyhedron {
    verts: Vec<Vertex>,
    edges: Vec<(u16,u16)>
}
impl From<Vec<Triangle>> for Polyhedron {
    fn from(values: Vec<Triangle>) -> Self {
       let verts: Vec<Rc<Vertex>> = values.iter().flat_map(|tri|{&tri.vertices}).collect();
       // println!("{}", verts);
       unimplemented!()
    }
}
impl Into<Vec<Triangle>> for Polyhedron {
    fn into(self) -> Vec<Triangle> {
        unimplemented!()
    }
}

impl Polyhedron {
}
