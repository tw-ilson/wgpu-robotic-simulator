use crate::graphics::{Vertex};
use std::borrow::Borrow;
use std::convert::{From, Into};
use std::cell::{RefCell, Ref, Cell};
use std::collections::{HashMap,HashSet};
use std::rc::Rc;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Rc<Vertex>; 3],
}
#[derive(Debug, Clone)]
pub struct Mesh {
    faces: Vec<Triangle>
}
impl Mesh {

}

#[derive(Debug, Clone)]
struct Polyhedron {
    verts: Vec<Vertex>,
    edges: Vec<(u16,u16)>
}
impl Polyhedron {
    fn check() {

    }
}
impl From<Mesh> for Polyhedron {
    fn from(mesh: Mesh) -> Self {
       // let mut edges: Vec<(u16,u16)> = vec![];
       // let i =0;
        let verts: Vec<Rc<Vertex>> = mesh.faces
            .iter()
            .flat_map(|tri| tri.vertices.iter().cloned())
            .dedup()
            .collect_vec();
        // let mut vert_indices: HashMap<Vertex, u16> = HashMap::new();
        // for (idx, vert) in verts.iter().enumerate() {
        //     vert_indices.insert(vert.borrow(), idx as u16);
        // }

       // get the correct set of edges
       unimplemented!()
    }
}
// impl Into<Vec<Triangle>> for Polyhedron {
//     fn into(self) -> Vec<Triangle> {
//         unimplemented!()
//     }
// }
//
