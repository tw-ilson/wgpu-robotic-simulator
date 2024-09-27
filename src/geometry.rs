use crate::graphics::Vertex;
use itertools::Itertools;
use rayon::prelude::*;
use std::convert::{From, Into};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
// use std::io::Read;
use bytemuck::{Pod, Zeroable};
use std::slice::Iter;
// use core::error::{Error, Result};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}
unsafe impl Zeroable for Triangle {}
unsafe impl Pod for Triangle {}
impl Triangle {
    fn new(verts: [glm::Vec3; 3], normal: glm::Vec3, color: glm::Vec3) -> Self {
        Self {
            vertices: [
                Vertex {
                    position: verts[0],
                    normal,
                    color,
                },
                Vertex {
                    position: verts[1],
                    normal,
                    color,
                },
                Vertex {
                    position: verts[2],
                    normal,
                    color,
                },
            ],
        }
    }
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
    pub tmatrix: glm::Mat4,
}
unsafe impl Pod for Transform {}
unsafe impl Zeroable for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            tmatrix: glm::Mat4x4::identity(),
        }
    }
}
impl std::ops::Mul<Vertex> for Transform {
    type Output = Vertex;
    fn mul(self, rhs: Vertex) -> Self::Output {
        Vertex {
            position: (self.tmatrix
                * glm::vec4(rhs.position.x, rhs.position.y, rhs.position.z, 1.0))
            .xyz(),
            normal: rhs.normal,
            color: rhs.color,
        }
    }
}
impl std::ops::Mul<Transform> for Transform {
    type Output = Transform;
    fn mul(self, rhs: Transform) -> Self::Output {
        Transform {
            tmatrix: self.tmatrix * rhs.tmatrix,
        }
    }
}
impl std::ops::Add<Transform> for Transform {
    type Output = Transform;
    fn add(self, rhs: Transform) -> Self::Output {
        Transform {
            tmatrix: self.tmatrix + rhs.tmatrix,
        }
    }
}
impl std::ops::Sub<Transform> for Transform {
    type Output = Transform;
    fn sub(self, rhs: Transform) -> Self::Output {
        Transform {
            tmatrix: self.tmatrix - rhs.tmatrix,
        }
    }
}

impl Transform {
    pub fn new(xyz: glm::Vec3, rpy: glm::Vec3) -> Self {
        let mut t = Self::default();
        t.translate(xyz);
        t.rotate_rpy(rpy);
        t
    }
    pub fn rotate_rpy(&mut self, rpy: glm::Vec3) {
        self.tmatrix = glm::rotate_x(&self.tmatrix, rpy[0]);
        self.tmatrix = glm::rotate_y(&self.tmatrix, rpy[1]);
        self.tmatrix = glm::rotate_z(&self.tmatrix, rpy[2]);
    }
    pub fn rotate(&mut self, axis: glm::Vec3, angle: f32) {
        self.tmatrix = glm::rotate(&self.tmatrix, angle, &axis);
    }
    pub fn translate(&mut self, xyz: glm::Vec3) {
        let t = glm::translate(&self.tmatrix, &xyz);
        self.tmatrix = t;
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..4 {
            write!(f, "[")?;
            for j in 0..4 {
                write!(f, "{:.3} ", self.tmatrix[(i, j)])?;
            }
            write!(f, "]")?;
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Polyhedron {
    // pub transform: Transform,
    pub verts: Vec<Vertex>,
    pub indices: Vec<u32>,
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
        //parallelize normal calculation
        self.faces.par_iter_mut().for_each(|tri| {
            let edge1 = tri.vertices[1].position - tri.vertices[0].position;
            let edge2 = tri.vertices[2].position - tri.vertices[1].position;
            let normal = glm::cross(&edge1, &edge2);
            let normal = glm::normalize(&normal);
            tri.vertices[0].normal = normal;
            tri.vertices[1].normal = normal;
            tri.vertices[2].normal = normal;
        })
    }
}

pub trait BoxMesh: Default {
    fn create_box(sz: glm::Vec3) -> Self;
}
pub trait CylinderMesh: Default {
    fn create_cylinder(r: f32, h: f32, nface: isize) -> Self;
}
pub trait PlaneMesh: Default {
    fn create_plane() -> Self;
}
pub trait SphereMesh: Default {
    fn create_sphere(r: f32, n_slices: usize, n_stacks: usize) -> Self;
}
impl BoxMesh for TriMesh {
    fn create_box(sz: glm::Vec3) -> Self {
        let [side1, side2, side3]: [glm::Vec3; 3];
        side1 = [sz[0], 0.0, 0.0].into();
        side2 = [0.0, sz[1], 0.0].into();
        side3 = [0.0, 0.0, sz[2]].into();

        let v1 = (sz*0.5) - /* 0.5 * */ (side1 + side2 + side3);
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
        let bottom = side3 * 0.5 - side3; // centre of base
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
impl SphereMesh for TriMesh {
    fn create_sphere(r: f32, n_slices: usize, n_stacks: usize) -> Self {
        let mut mesh = TriMesh::default();
        let mut verts: Vec<glm::Vec3> = Vec::with_capacity(n_stacks * n_slices);

        // add top vertex
        let v0: glm::Vec3 = [0.0, r, 0.0].into();
        verts.push(v0);

        use std::f32::consts::PI;
        // generate vertices per stack / slice
        for i in 0..(n_stacks - 1) {
            let phi = PI * (i as f32 + 1.0) / (n_stacks as f32);
            for j in 0..n_slices {
                let theta = 2.0 * PI * (j as f32) / (n_slices as f32);
                let x = r * phi.sin() * theta.cos();
                let y = r * phi.cos();
                let z = r * phi.sin() * theta.sin();
                verts.push([x, y, z].into());
            }
        }

        // add bottom vertex
        let v1: glm::Vec3 = [0.0, -r, 0.0].into();
        verts.push(v1);

        // add top / bottom triangles
        for i in 0..n_slices {
            let i0 = i + 1;
            let i1 = (i + 1) % n_slices + 1;
            mesh.add_triangle([v0, verts[i1], verts[i0]]);
            let i0 = i + n_slices * (n_stacks - 2) + 1;
            let i1 = (i + 1) % n_slices + n_slices * (n_stacks - 2) + 1;
            mesh.add_triangle([v1, verts[i0], verts[i1]]);
        }

        // add quads per stack / slice
        for j in 0..(n_stacks - 2) {
            let j0 = j * n_slices + 1;
            let j1 = (j + 1) * n_slices + 1;
            for i in 0..n_slices {
                let i0 = j0 + i;
                let i1 = j0 + (i + 1) % n_slices;
                let i2 = j1 + (i + 1) % n_slices;
                let i3 = j1 + i;
                mesh.add_rectangle([verts[i0], verts[i1], verts[i2], verts[i3]]);
            }
        }

        mesh
    }
}
impl PlaneMesh for TriMesh {
    fn create_plane() -> Self {
        static SIZE: f32 = 100.0;
        let mut mesh = TriMesh::default();
        mesh.add_rectangle([
            glm::vec3(-SIZE, -SIZE, 0.0),
            glm::vec3(SIZE, -SIZE, 0.0),
            glm::vec3(SIZE, SIZE, 0.0),
            glm::vec3(-SIZE, SIZE, 0.0),
        ]);
        mesh
    }
}

//entirely taken from pk-stl
pub fn parse_binary_stl(bytes: &[u8]) -> TriMesh {
    let mut data = bytes.iter();

    let _header: Vec<u8> = data.by_ref().take(80).map(|val| *val).collect();
    // let _header: String = String::from_utf8_lossy(&_header).trim_end_matches("\0").to_string();

    let triangle_count = {
        let mut raw = [0; 4];

        for i in 0..4 {
            raw[i] = match data.next() {
                Some(val) => *val,
                None => panic!(),
            }
        }

        u32::from_le_bytes(raw)
    };

    let mut faces: Vec<Triangle> = Vec::with_capacity(triangle_count as usize);

    for _ in 0..(triangle_count as usize) {
        let normal = glm::Vec3::from(read_f32_triplet(&mut data).unwrap());
        let vert_a = glm::Vec3::from(read_f32_triplet(&mut data).unwrap());
        let vert_b = glm::Vec3::from(read_f32_triplet(&mut data).unwrap());
        let vert_c = glm::Vec3::from(read_f32_triplet(&mut data).unwrap());

        let _ = data.next();
        let _ = data.next();

        let tri = Triangle::new(
            [vert_a, vert_b, vert_c],
            normal,
            glm::Vec3::default(), // normal
        );
        // println!("{:#?}", tri);
        faces.push(tri)
    }

    TriMesh { faces }
}

fn read_f32_triplet<'a>(data: &mut Iter<'a, u8>) -> Result<[f32; 3], String> {
    Ok([read_f32(data)?, read_f32(data)?, read_f32(data)?])
}

fn read_f32<'a>(data: &mut Iter<'a, u8>) -> Result<f32, String> {
    let mut raw = [0; 4];
    for item in &mut raw {
        *item = match data.next() {
            Some(val) => *val,
            None => return Err("Invalid triangle count byte sequence".into()),
        };
    }

    Ok(f32::from_le_bytes(raw))
}

fn parse_ascii_stl(fstring: String) -> TriMesh {
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
                            get_vert(&mut lines[k + 2].split_whitespace()).into(),
                            get_vert(&mut lines[k + 3].split_whitespace()).into(),
                            get_vert(&mut lines[k + 4].split_whitespace()).into(),
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
fn parse_stl(fname: String) -> TriMesh {
    // let mut file = std::fs::File::open(fname).expect("Unable to open file");
    let bytes = std::fs::read(fname).expect("unable to read file");
    if &bytes[0..6] == b"solid " {
        // parse_ascii_stl(std::fs::read_to_string(fname).expect("could not read file"))
        parse_ascii_stl(String::from_utf8(bytes).expect("could not convert to utf8"))
    } else {
        parse_binary_stl(bytes.as_slice())
    }
}

fn parse_obj(fname: String) -> TriMesh {
    let file = match File::open(fname) {
        Ok(file) => file,
        Err(_) => {
            println!("FILE not found!");
            std::process::exit(-1);
        }
    };
    // enum State {
    //     Wait,
    //     Vertex,
    //     Normal,
    //     Face,
    // }

    let reader = BufReader::new(file);
    // let mut state = State::Wait;
    let mut vertices: Vec<glm::Vec3> = Vec::new();
    let mut normals: Vec<glm::Vec3> = Vec::new();
    let mut faces: Vec<Triangle> = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            let mut tokens = line.split_whitespace();
            if let Some(token) = tokens.next() {
                match token {
                    "v" => {
                        let x = tokens.next().unwrap().parse::<f32>().unwrap();
                        let y = tokens.next().unwrap().parse::<f32>().unwrap();
                        let z = tokens.next().unwrap().parse::<f32>().unwrap();
                        vertices.push(glm::vec3(x, y, z));
                    }
                    "vn" => {
                        let x = tokens.next().unwrap().parse::<f32>().unwrap();
                        let y = tokens.next().unwrap().parse::<f32>().unwrap();
                        let z = tokens.next().unwrap().parse::<f32>().unwrap();
                        normals.push(glm::vec3(x, y, z));
                    }
                    "f" => {
                        let mut face = Triangle {
                            vertices: [Vertex::default(); 3],
                        };
                        let re =
                            regex::Regex::new(r"v?((?:[0-9])*)\/\/(?:vn)?((?:[0-9])*)").unwrap();

                        for (k, token) in tokens.enumerate().take(3) {
                            if let Some(captures) = re.captures(token) {
                                let vidx = captures[1].parse::<usize>().unwrap_or_default() - 1;
                                let nidx = captures[2].parse::<usize>().unwrap_or_default() - 1;
                                face.vertices[k].position = vertices.get(vidx).unwrap().clone();
                                face.vertices[k].normal = normals.get(nidx).unwrap().clone();
                            }
                        }
                        faces.push(face);
                    }
                    "vt" => continue,
                    "mtllib" => continue,
                    "usemtl" => continue,
                    "s" => continue,
                    "o" => continue,
                    _ => {
                        if token.trim().starts_with("#") {
                            continue;
                        } else {
                            eprintln!("Unexpected token {} -- possibly file contains unsupported features", token);
                        }
                    }
                }
            }
        }
    }
    TriMesh { faces }
}

impl From<MeshType> for TriMesh {
    fn from(mesh_type: MeshType) -> Self {
        match mesh_type {
            MeshType::STL(fname) => parse_stl(fname),
            MeshType::OBJ(fname) => parse_obj(fname),
            _ => panic!("type unsupported"),
        }
    }
}

impl Default for Polyhedron {
    // empty polyhedron
    fn default() -> Self {
        Self {
            verts: vec![],
            indices: vec![],
        }
    }
}

impl Polyhedron {
    pub fn verts(&self) -> &Vec<Vertex> {
        &self.verts
    }
    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
    pub fn set_color(&mut self, color: glm::Vec3) {
        //parallelize coloring verts
        self.verts.par_iter_mut().for_each(|v| (*v).color = color);
    }
    pub fn scale(&mut self, factor: f32) {
        //parallelize scaling verts
        self.verts
            .par_iter_mut()
            .for_each(|v| (*v).position *= factor);
    }
    pub fn scale_xyz(&mut self, factor: glm::Vec3) {
        self.verts
            .par_iter_mut() // parallelize
            .for_each(|v| v.position = glm::diagonal3x3(&factor) * v.position);
    }
}

impl From<String> for Polyhedron {
    fn from(value: String) -> Self {
        match value.split(".").last().unwrap().to_lowercase().as_str() {
            "stl" => Polyhedron::from(TriMesh::from(MeshType::STL(value))),
            "obj" => Polyhedron::from(TriMesh::from(MeshType::OBJ(value))),
            _ => unimplemented!(),
        }
    }
}

pub trait OptimizeMesh<T> {
    fn optimize(mesh: T) -> Self;
}
impl OptimizeMesh<TriMesh> for Polyhedron {
    // create efficient index buffer -- adds overhead
    fn optimize(mesh: TriMesh) -> Self {
        // mesh.calculate_normals();
        let verts: Vec<Vertex> = mesh
            .faces
            .iter()
            .flat_map(|tri| tri.vertices.iter().cloned())
            .dedup()
            .collect_vec();

        let indices: Vec<u32> = mesh
            .faces
            .iter()
            .flat_map(|tri| {
                let mut indices: Vec<u32> = Vec::new();
                for i in 0..3 {
                    let idx1 = verts
                        .iter()
                        .position(|&v_b| v_b == tri.vertices[i])
                        .unwrap();
                    indices.push(idx1 as u32);
                }
                indices
            })
            .collect();
        let poly = Self { verts, indices };
        poly
    }
}

impl From<TriMesh> for Polyhedron {
    fn from(mut mesh: TriMesh) -> Self {
        mesh.calculate_normals();
        Self {
            indices: (0..3 * mesh.faces.len() as u32).collect(),
            verts: bytemuck::cast_vec::<Triangle, Vertex>(mesh.faces),
        }
    }
}
