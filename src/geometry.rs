use crate::graphics::Vertex;
use itertools::Itertools;
use std::{
    convert::{From, Into},
};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}

#[non_exhaustive]
pub enum MeshType {
    STL(String),
    OBJ(String),
}

#[derive(Default, Debug, Clone)]
pub struct TriMesh {
    faces: Vec<Triangle>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    tmatrix: glm::Mat4,
}
unsafe impl bytemuck::Pod for Transform {}
unsafe impl bytemuck::Zeroable for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self { tmatrix: glm::Mat4x4::identity() }
    }
}

impl std::ops::Add<Transform> for Transform {
    type Output = Transform;
    fn add(self, rhs: Transform) -> Self::Output {
        Transform { tmatrix: self.tmatrix + rhs.tmatrix }
    }
}
impl std::ops::Sub<Transform> for Transform {
    type Output = Transform;
    fn sub(self, rhs: Transform) -> Self::Output {
        Transform { tmatrix: self.tmatrix - rhs.tmatrix }
    }
}

impl Transform {
    fn new(xyz: glm::Vec3, rpy: glm::Vec3) -> Self {
        let mut t = Self::default();
        t.rotate_rpy(rpy);
        t.translate(xyz);
        t
    }
    fn rotate_rpy(&mut self, rpy: glm::Vec3) {
        self.tmatrix += glm::rotate_x(&self.tmatrix, rpy[0]);
        self.tmatrix += glm::rotate_y(&self.tmatrix, rpy[1]);
        self.tmatrix += glm::rotate_z(&self.tmatrix, rpy[2]);
    }
    fn rotate(&mut self, axis: glm::Vec3, angle: f32) {
        self.tmatrix += glm::rotate(&self.tmatrix, angle, &axis);
    }
    fn translate(&mut self, xyz: glm::Vec3) {
        self.tmatrix += glm::translate(&self.tmatrix, &xyz);
    }
}

#[derive(Debug, Clone)]
pub struct Polyhedron {
    transform: Transform,
    pub verts: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl TriMesh {
    pub fn add_triangle(&mut self, v: [glm::Vec3; 3]) {
        self.faces.push(Triangle {
            vertices: [v[0].into(), v[1].into(), v[2].into()],
        })
    }
    pub fn add_rectangle(&mut self, quad: [glm::Vec3; 4]) {
        self.faces.push(Triangle {
            vertices: [quad[0].into(), quad[1].into(), quad[2].into()],
        });
        self.faces.push(Triangle {
            vertices: [quad[0].into(), quad[2].into(), quad[3].into()],
        });
    }
    pub fn calculate_normals(&mut self) {
        for tri in self.faces.iter_mut() {
            let edge1 = tri.vertices[1].position - tri.vertices[0].position;
            let edge2 = tri.vertices[2].position - tri.vertices[1].position;
            let normal = glm::cross(&edge1, &edge2);
            tri.vertices[0].normal = normal;
            tri.vertices[1].normal = normal;
            tri.vertices[2].normal = normal;
        }
    }
}

pub trait BoxMesh: Default {
    fn create_box(sz: glm::Vec3) -> Self;
}
pub trait CylinderMesh: Default {
    fn create_cylinder(r: f32, h: f32, nface: isize) -> Self;
}
impl BoxMesh for TriMesh {
    fn create_box(sz: glm::Vec3) -> Self {
        let [side1, side2, side3]: [glm::Vec3; 3];
        side1 = [sz[0], 0.0, 0.0].into();
        side2 = [0.0, sz[1], 0.0].into();
        side3 = [0.0, 0.0, sz[2]].into();

        let v1 = 0.5 * (side1 + side2 + side3);
        let v2 = v1 + side1;
        let v3 = v2 + side2;
        let v4 = v3 - side1;
        let v5 = v1 + side3;
        let v6 = v2 + side3;
        let v7 = v3 + side3;
        let v8 = v4 + side3;
        let mut tris = TriMesh::default();
        tris.add_rectangle([v1, v2, v6, v5]);
        tris.add_rectangle([v2, v3, v7, v6]);
        tris.add_rectangle([v3, v4, v8, v7]);
        tris.add_rectangle([v4, v1, v5, v8]);
        tris.add_rectangle([v1, v4, v3, v2]);
        tris.add_rectangle([v5, v6, v7, v8]);
        return tris;
    }
}
impl CylinderMesh for TriMesh {
    fn create_cylinder(r: f32, h: f32, nface: isize) -> TriMesh {
        let [side1, side2, side3]: [glm::Vec3; 3];
        side1 = [r, 0.0, 0.0].into();
        side2 = [0.0, r, 0.0].into();
        side3 = [0.0, 0.0, h].into();
        let bottom = 0.5 * side3; // centre of base
        let top = bottom + side3; // centre of top

        use std::f32::consts::PI;
        let dtheta: f32 = 2.0 * PI / (nface as f32);
        let [mut v1, mut v2, mut v3, mut v4]: [glm::Vec3; 4];
        v2 = bottom + side1;
        v3 = v2 + side3;
        let mut mesh = TriMesh::default();
        for n in 1..=nface {
            let theta: f32 = (n as f32) * dtheta;
            v1 = v2;
            v4 = v3;
            v2 = bottom + theta.cos() * side1 + theta.sin() * side2;
            v3 = v2 + side3;
            mesh.add_rectangle([v1, v2, v3, v4]); // add sides as a series of rectangles
            mesh.add_triangle([v2, v1, bottom]); // add triangles for bottom
            mesh.add_triangle([v4, v3, top]); // add triangles for top
        }
        return mesh;
    }
}

fn parse_stl(fstring: String) -> TriMesh {
    let lines: Vec<&str> = {
        let mut lines = fstring.lines();
        match lines.next() {
            Some(s) => {
                if !s.trim().starts_with("solid") {
                    panic!()
                }
            }
            _ => panic!(),
        }
        lines
    }
    .collect_vec();
    TriMesh {
        faces: {
            let mut k = 0;
            lines
                .iter()
                .filter_map(|line| {
                    fn get_vert(toks: &mut std::str::SplitWhitespace) -> glm::Vec3 {
                        let t = toks.next();
                        assert!(Some("vertex") == t || Some("normal") == t);
                        [
                            toks.next().unwrap().parse::<f32>().unwrap(),
                            toks.next().unwrap().parse::<f32>().unwrap(),
                            toks.next().unwrap().parse::<f32>().unwrap(),
                        ]
                        .into()
                    }
                    let mut split = line.split_whitespace();
                    let r = if split.next().unwrap() == "facet" {
                        let normal = get_vert(&mut split);
                        let mut vertices: [Vertex; 3] = [
                            get_vert(&mut lines.clone()[k + 2].split_whitespace()).into(),
                            get_vert(&mut lines.clone()[k + 3].split_whitespace()).into(),
                            get_vert(&mut lines.clone()[k + 4].split_whitespace()).into(),
                        ];
                        vertices[0].normal = normal;
                        vertices[1].normal = normal;
                        vertices[2].normal = normal;
                        Some(Triangle { vertices })
                    } else {
                        None
                    };
                    k += 1;
                    r
                })
                .collect()
        },
    }
}

impl From<MeshType> for TriMesh {
    fn from(mesh_type: MeshType) -> Self {
        match mesh_type {
            MeshType::STL(fname) => {
                parse_stl(std::fs::read_to_string(fname).expect("could not read file"))
            }
            // OBJ(fstring) -> parse_obj(fstring),
            _ => panic!("type unsupported"),
        }
    }
}

impl Default for Polyhedron {
    // empty polyhedron
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            verts: vec![],
            indices: vec![],
        }
    }
}

impl Polyhedron {
    pub fn from_fast(mesh: TriMesh) -> Self {
        Self {
            transform: Transform::default(),
            verts: mesh.faces.iter().flat_map(|tri| tri.vertices).collect(),
            indices: (0..mesh.faces.len() as u16)
                .flat_map(|fi| {
                    let k: u16 = fi * 3;
                    k..k + 3
                })
                .collect(),
        }
    }
    // pub fn update(&mut self) {
    //
    // }
    pub fn scale(&mut self, factor: f32) {
        for v in self.verts.iter_mut() {
            (*v).position *= factor;
        }
    }
}

impl From<String> for Polyhedron {
    fn from(value: String) -> Self {
        match value.split(".").last().unwrap().to_lowercase().as_str() {
            "stl" => Polyhedron::from(TriMesh::from(MeshType::STL(value))),
            _ => unimplemented!(),
        }
    }
}

impl From<TriMesh> for Polyhedron {
    // create efficient index buffer -- adds overhead
    fn from(mut mesh: TriMesh) -> Self {
        mesh.calculate_normals();
        let verts: Vec<Vertex> = mesh
            .faces
            .iter()
            .flat_map(|tri| tri.vertices.iter().cloned())
            .dedup()
            .collect_vec();
        let indices: Vec<u16> = mesh
            .faces
            .iter()
            .flat_map(|tri| {
                let mut indices: Vec<u16> = Vec::new();
                for i in 0..3 {
                    let idx1 = verts
                        .iter()
                        .position(|&v_b| v_b == tri.vertices[i])
                        .unwrap();
                    indices.push(idx1 as u16);
                }
                indices
            })
            .collect();
        let mut poly = Self {
            transform: Transform::default(),
            verts,
            indices,
        };
        poly
    }
}
