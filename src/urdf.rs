use crate::geometry::{BoxMesh, CylinderMesh, SphereMesh, Polyhedron, TriMesh, Transform, self, MeshBuffer};
use crate::wgpu_program::WGPUGraphics;
use glm;
use itertools::Itertools;
use std::str::FromStr;
use xml::reader::{XmlEvent, XmlEvent::*};
use xml::EventReader;

#[derive(Default, Debug, Copy, Clone)]
pub struct Origin {
    xyz: glm::Vec3,
    rpy: Option<glm::Vec3>,
}

impl From<Origin> for Transform {
    fn from(value: Origin) -> Self {
        Transform::new(value.xyz, value.rpy.unwrap_or_default())
    }
}

#[derive(Default, Debug, Clone)]
pub struct InertialBody {
    pub transform: Transform,
    pub mass: f32,
    pub ixx: f32,
    pub iyy: f32,
    pub izz: f32,
    pub ixy: f32,
    pub ixz: f32,
    pub iyz: f32,
}

#[derive(Default, Debug, Clone)]
pub struct VisualBody {
    pub geometry: Polyhedron,
    pub material: Option<String>,
}
#[derive(Default, Debug, Clone)]
pub struct CollisionBody {
    pub transform: Transform,
    pub geometry: Polyhedron,
}

#[derive(Default, Debug, Clone)]
pub struct Link {
    pub link_name: String,
    pub visual: VisualBody,
    pub inertial: InertialBody,
    pub collision: CollisionBody,
}

#[derive(Debug, Copy, Clone)]
pub enum JointType {
    Revolute,
    Fixed,
    Continuous,
    Prismatic,
    Floating,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct JointLimits {
    effort: f32,
    velocity: f32,
    lower: f32,
    upper: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct JointDynamics {
    damping: f32,
    friction: f32,
}

#[derive(Debug, Clone)]
pub struct Joint {
    joint_name: String,
    joint_type: JointType,
    parent: usize, // index of the link
    child: usize,  // index of the link
    // origin: Option<Origin>,
    transform: Transform,
    axis: Option<glm::Vec3>, // axis in joint frame
    limits: Option<JointLimits>,
    dynamics: Option<JointDynamics>
}
#[derive(Default, Debug, Clone)]
pub struct RobotDescriptor {
    pub name: Option<String>,
    pub links: Vec<Link>,
    pub joints: Vec<Joint>,
}

type ParseRobotError = Box<dyn std::error::Error>;

//gets position, rotation from origin element
fn parse_3f(s: &str) -> Result<glm::Vec3, ParseRobotError> {
    let v: [f32; 3] = s
        .split_whitespace()
        .map(|ns| ns.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>().unwrap()
        .try_into()
        .unwrap();
    Ok(v.into())
}
fn parse_4f(s: &str) -> Result<glm::Vec4, ParseRobotError> {
    let v: [f32; 4] = s
        .split_whitespace()
        .map(|ns| ns.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>().unwrap()
        .try_into()
        .unwrap();
    Ok(v.into())
}
fn parse_origin(origin_event: XmlEvent) -> Result<Origin, ParseRobotError> {
    if let XmlEvent::StartElement {
        attributes,
        ..
    } = origin_event
    {
        let xyz_attr = attributes.iter().find(|&a| a.name.local_name == "xyz").ok_or("expected attribute xyz").unwrap();
        let xyz = parse_3f(&xyz_attr.value).unwrap();
        let rpy_attr = attributes.iter().find(|&a| a.name.local_name == "rpy");
        let rpy = if let Some(attr) = rpy_attr {
            let value = parse_3f(&attr.value).ok();
            value
        } else {
            None
        };
        Ok(Origin { xyz, rpy })
    } else {
        unreachable!();
    }
}


fn parse_link_geometry(
    xml_parser: &mut EventReader<&[u8]>,
    mut link: Link,
) -> Result<Link, ParseRobotError> {
    loop {
        let event = xml_parser.next();
        match event.unwrap() {
            StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "mesh" => {
                    let fname = attributes
                        .iter()
                        .find(|&a| a.name.local_name == "filename")
                        .ok_or("expected file name attribute").unwrap()
                        .value
                        .to_owned();
                    // let fstring = std::fs::read_to_string(fname).unwrap();
                    link.visual.geometry = Polyhedron::from(fname.to_owned());
                    if let Some(scale) = attributes
                        .iter()
                        .find(|&a| a.name.local_name == "scale") {
                            link.visual.geometry.scale_xyz(parse_3f(&scale.value).unwrap());
                        }
                }
                "box" | "cylinder" | "sphere" => match name.local_name.as_str() {
                    "box" => {
                        let size_attr = attributes.get(0).ok_or("expected sized").unwrap();
                        if size_attr.name.local_name == "size" {
                            let size = parse_3f(&size_attr.value).unwrap();
                            link.visual.geometry = TriMesh::create_box(size).into();
                        } else {
                            return Err("box requires size attribute".into())
                        }
                    }
                    "cylinder" => {
                        let l = attributes
                            .iter()
                            .find(|&a| a.name.local_name == "length")
                            .ok_or("cylinder requires length").unwrap()
                            .value
                            .parse::<f32>().unwrap();
                        let r = attributes
                            .iter()
                            .find(|&a| a.name.local_name == "radius")
                            .ok_or("cylinder requires radius").unwrap()
                            .value
                            .parse::<f32>().unwrap();
                        link.visual.geometry = TriMesh::create_cylinder(r, l, 30).into();
                    }
                    "sphere" => {
                        let r = attributes
                            .iter()
                            .find(|&a| a.name.local_name == "radius")
                            .ok_or("sphere requires radius").unwrap()
                            .value
                            .parse::<f32>().unwrap();
                        link.visual.geometry = TriMesh::create_sphere(r,20, 20).into();
                    }
                    _ => return Err("unknown element".into()),
                },
                _ => {return Err("unknown element".into())}
            },
            EndElement { name } => {
                if name.local_name == "geometry" {
                    return Ok(link);
                }
            }
            _ => {}
        }
    }
}

fn parse_link_visual(
    xml_parser: &mut EventReader<&[u8]>,
    mut link: Link,
    materials: &mut Vec<Material>,
) -> Result<Link, ParseRobotError> {
    let mut transform: Option<Transform> = None;
    loop {
        let event = xml_parser.next();
        match event.clone().unwrap() {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "origin" => {
                    let Origin{xyz, rpy} = parse_origin(event.unwrap()).unwrap();
                    transform = Some(Transform::new(xyz, rpy.unwrap_or_default()));
                    },
                "geometry" => link = parse_link_geometry(xml_parser, link).unwrap(),
                "material" => if link.visual.material.is_none() {
                    let mat_name = &attributes
                        .iter()
                        .find(|&a| a.name.local_name == "name")
                        .ok_or("material requires name").unwrap()
                        .value
                        .to_owned();
                    link.visual.material = Some(mat_name.to_owned());
                    // if let Some(mat) = materials.iter().find(|m| m.name == *mat_name) {
                    //     link.geometry.set_color(mat.color);
                    // } else {
                    if let Ok(mat) = parse_material(xml_parser, mat_name.to_owned()) {
                        materials.push(mat);
                    }
                    // }
                },
                _ => {}
            },
            EndElement { name } => {
                link.visual.geometry.transform = transform.unwrap_or_default();
                if name.local_name == "visual" {
                    return Ok(link);
                }
            }
            _ => {}
        }
    }
}

fn parse_link_collision(
    xml_parser: &mut EventReader<&[u8]>,
    mut link: Link,
) -> Result<Link, ParseRobotError> {
    loop {
        let event = xml_parser.next();
        match event.clone().unwrap() {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                _ => {}
            },
            EndElement { name } => {
                if name.local_name == "collision" {
                    return Ok(link);
                }
            }
            _ => {}
        }
    }
}
fn parse_link_inertial(
    xml_parser: &mut EventReader<&[u8]>,
    mut link: Link,
) -> Result<Link, ParseRobotError> {
    let mut origin: Option<Origin> = None;
    let mut mass: Option<f32> = None;
    let mut inertia: Option<[f32;6]> = None;
    loop {
        let event = xml_parser.next();
        match event.clone().unwrap() {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "origin" => {origin = parse_origin(event.unwrap()).ok()},
                "mass" => { mass = attributes.get(0).unwrap().value.parse::<f32>().ok()}
                "inertia" => { 
                     inertia = Some([
                        attributes.iter().find(|a| a.name.local_name == "ixx").unwrap().value.parse::<f32>().unwrap(),
                        attributes.iter().find(|a| a.name.local_name == "iyy").unwrap().value.parse::<f32>().unwrap(),
                        attributes.iter().find(|a| a.name.local_name == "izz").unwrap().value.parse::<f32>().unwrap(),
                        attributes.iter().find(|a| a.name.local_name == "ixy").unwrap().value.parse::<f32>().unwrap(),
                        attributes.iter().find(|a| a.name.local_name == "ixz").unwrap().value.parse::<f32>().unwrap(),
                        attributes.iter().find(|a| a.name.local_name == "iyz").unwrap().value.parse::<f32>().unwrap(),
                        ]);
                },
                _ => {}
            },
            EndElement { name } => {
                if name.local_name == "inertial" {
                    if let Some(mass) = mass {
                        if let Some([ixx, iyy, izz, ixy, ixz, iyz]) = inertia {
                            link.inertial = InertialBody { transform: origin.unwrap_or_default().into(), mass, ixx, iyy, izz, ixy, ixz, iyz };
                            return Ok(link);
                        } else {panic!("inertial body requires moments of inertia!")}  
                    } else {panic!("inertial body requires mass!")}
                }
            }
            _ => {}
        }
    }
}

fn parse_link(
    xml_parser: &mut EventReader<&[u8]>,
    link_name: String,
    materials: &mut Vec<Material>,
) -> Result<Link, ParseRobotError> {
    let mut link = Link::default();
    link.link_name = link_name;
    loop {
        let event = xml_parser.next();
        match event.clone().unwrap() {
            StartElement {
                name, ..
            } => {
                match name.local_name.as_str() {
                    "visual" => {
                        link = parse_link_visual(xml_parser, link, materials).unwrap();
                    }
                    "inertial" => {link = parse_link_inertial(xml_parser, link).unwrap()},
                    "collision" => {link = parse_link_collision(xml_parser, link).unwrap()},
                    _ => {}
                }
            }
            EndElement { name } => {
                if name.local_name == "link" {
                    return Ok(link);
                }
            }
            _ => {} 
        }
    }
}

pub fn parse_joint(
    xml_parser: &mut EventReader<&[u8]>,
    joint_name: String,
    joint_type: JointType,
    links: &Vec<Link>,
) -> Result<Joint, ParseRobotError> {
    let mut parent_name: Option<String> = None;
    let mut child_name: Option<String> = None;
    let mut origin: Option<Origin> = None;
    let mut axis: Option<glm::Vec3> = None;
    let mut limits: Option<JointLimits> = None;
    let mut dynamics: Option<JointDynamics> = None;

    loop {
        let event = xml_parser.next();
        match event.clone().unwrap() {
            StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "parent" => {
                    parent_name = Some(
                        attributes
                            .get(0)
                            .expect("parent requires name")
                            .value
                            .clone(),
                    )
                }
                "child" => {
                    child_name = Some(
                        attributes
                            .get(0)
                            .expect("child requires name")
                            .value
                            .clone(),
                    )
                }
                "origin" => origin = Some(parse_origin(event.clone().unwrap()).unwrap()),
                "axis" => {
                    let attr = attributes.get(0).ok_or("expected attribute xyz").unwrap();
                    axis = Some(parse_3f(&attr.value).unwrap());
                }
                "limit" => {
                    let (mut effort, mut lower, mut upper, mut velocity) = (0., 0., 0., 0.);
                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            "effort" => effort = attr.value.parse::<f32>().unwrap(),
                            "lower" => lower = attr.value.parse::<f32>().unwrap(),
                            "upper" => upper = attr.value.parse::<f32>().unwrap(),
                            "velocity" => velocity = attr.value.parse::<f32>().unwrap(),
                            _ => {panic!("unkown attribute in limit")}
                        }
                    }
                    limits = Some(JointLimits {
                        effort,
                        velocity,
                        lower,
                        upper,
                    });
                },
                "dynamics" => {
                    let (mut damping, mut friction) = (0.0, 0.0);
                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            "damping" => damping = attr.value.parse::<f32>().unwrap(),
                            "friction" => friction = attr.value.parse::<f32>().unwrap(),
                            _ => {panic!("unknown attribute in dynamics")}
                        }
                    }
                    dynamics = Some(JointDynamics { damping, friction});
                }
                _ => {
                    eprintln!("{}", name.local_name.as_str());
                    return Err("possibly unsupported feature".into())
                }
            },
            EndElement { name } => {
                if name.local_name == "joint" {
                    break;
                }
            },
            Whitespace(..) => {},
            _ => {
            }
        }
    }
    let (parent, child);
    let p_name = parent_name.ok_or("parent element is required!").unwrap();
    parent = links
        .iter()
        .position(|l| l.link_name == p_name)
        .ok_or(format!("no known link with name {}", p_name)).unwrap();
    let c_name = child_name.ok_or("child element is required!").unwrap();
    child = links
        .iter()
        .position(|l| l.link_name == c_name)
        .ok_or(format!("no known link with name {}", c_name)).unwrap();
    let transform = if let Some(Origin { xyz, rpy }) = origin {
        Transform::new(xyz, rpy.unwrap_or_default())
    } else {
        Transform::default()
    };
    Ok(Joint {
        joint_name,
        joint_type,
        parent,
        child,
        transform,
        axis,
        limits,
        dynamics
    })
}

#[derive(Debug, Default, PartialEq, Clone)]
struct Material {
    name: String,
    color: glm::Vec3,
}
fn parse_material(xml_parser: &mut EventReader<&[u8]>, material_name: String) -> Result<Material, ParseRobotError> {
    let color: glm::Vec3;
    loop {
        let event = xml_parser.next().unwrap();
        match event {
            StartElement { name, attributes, .. } => {
                match name.local_name.as_str() {
                    "color" => {
                        let attr = attributes.iter().find(|&a| a.name.local_name == "rgba").ok_or("color must have rgba value").unwrap();
                        color = parse_4f(&attr.value).unwrap().xyz();
                        return Ok(Material{name:material_name, color})
                    },
                    "texture" => {unimplemented!()}
                    _ => {return Err("unknown element in material".into())}
                }
            },
            EndElement { name } => if name.local_name == "material" {
                return Err("could not parse material".into());
            },
            Whitespace(..) => {},
            _ => {
                return Err("could not parse material".into());
            }
        }
    }
}
fn parse_robot(
    mut xml_parser: EventReader<&[u8]>,
    robot_name: Option<String>,
) -> Result<RobotDescriptor, ParseRobotError> {
    let mut links = Vec::new();
    let mut joints = Vec::new();
    let mut materials = Vec::<Material>::new();
    let mut attr_name: String;
    loop {
        let event = xml_parser.next();
        match event.unwrap() {
            StartElement {
                name, attributes, ..
            } => {
                let attr = attributes.get(0).ok_or("link requires name").unwrap();
                assert!(attr.name.local_name == "name");
                attr_name = attr.value.to_owned();

                match name.local_name.as_str() {
                    "link" => links.push(parse_link(&mut xml_parser, attr_name, &mut materials).unwrap()),
                    "joint" => {
                        let joint_type: JointType;
                        if let Some(attr) = attributes.get(1) {
                            assert!(attr.name.local_name == "type");
                            match attr.value.as_str() {
                                "fixed" => joint_type = JointType::Fixed,
                                "revolute" => joint_type = JointType::Revolute,
                                "continuous" => joint_type = JointType::Continuous,
                                "prismatic" => joint_type = JointType::Prismatic,
                                "floating" => joint_type = JointType::Floating,
                                _ => {
                                    return Err("unrecognized joint type".into())
                                }
                            }
                        } else {
                            return Err("joint requires type attribute".into())
                        }
                        joints.push(parse_joint(&mut xml_parser, attr_name, joint_type, &links).unwrap())
                    }
                    "material" => materials.push(parse_material(&mut xml_parser, attr_name).unwrap()),
                    _ => return Err(format!("unexpected element name! \"{}\"", name.local_name).into()),
                }
            }
            EndElement { name } => {
                if name.local_name == "robot" {
                    break;
                }
            }
            _ => {}
        }
    }

    //setup colors
    for mat in materials {
        for link in links.iter_mut() {
            // println!("{:?}", link.visual.material);
            if link.visual.material.clone().is_some_and(|mn| mn == mat.name) {
                // println!("{:?}", mat);
                link.visual.geometry.set_color(mat.color);
            }
        }
    }

    return Ok(RobotDescriptor {
        name: robot_name,
        links,
        joints,
    });
}

impl FromStr for RobotDescriptor {
    type Err = ParseRobotError;
    fn from_str(s: &str) -> Result<RobotDescriptor, ParseRobotError> {
        let mut xml_parser = EventReader::from_str(s);
        let mut robot_name: Option<String> = None;
        let event = xml_parser.next();
        match event.unwrap() {
            StartDocument { .. } => {}
            _ => return Err("Is this a valid XML URDF file.unwrap()".into()),
        }
        let event = xml_parser.next();
        match event.unwrap() {
            StartElement {
                name, attributes, ..
            } => {
                assert!(name.local_name == "robot");
                if let Some(attr) = attributes.get(0) {
                    if attr.name.local_name == "name" {
                        robot_name = Some(attr.value.clone());
                    }
                };
                return parse_robot(xml_parser, robot_name);
            }
            _ => Err("expected robot element as first element".into()),
        }
    }
}

impl RobotDescriptor {
    fn walk_children(&self, cur_link: &Link) -> Vec<(usize, Transform)> {
        self.joints
            .iter()
            .filter(|j| self.links[j.parent].link_name == cur_link.link_name)
            .map(|j| {
                let tf = cur_link.visual.geometry.transform * j.transform * self.links[j.child].visual.geometry.transform;
                (j.child, tf)
            })
            .collect_vec()
    }

    pub fn update(&mut self, theta: Vec<f32>) {
        unimplemented!();
    }
    // Walk the DAG
    pub fn build(&mut self) {
        //next, setup transforms
        let base_link = self.links.get(0).expect("No links found.unwrap()");
        let mut child_transforms = self.walk_children(base_link);
        loop {
            let mut queue: Option<Vec<(usize, Transform)>> = None;
            for (c_id, c_tf) in &child_transforms {
                self.links[*c_id].visual.geometry.transform = *c_tf;
                queue = Some(self.walk_children(&self.links[*c_id]));
            }
            if let Some(qlist) = queue {
                child_transforms = qlist;
            } else {
                break;
            }
        }
        self.links.iter_mut()
        .for_each(|l| l.visual.geometry.update_base())
    }
    // pub fn mesh_list(&self) -> Vec<&Polyhedron> {
    //
    // }
}

pub trait RobotGraphics {
    fn robot_create_buffers(&mut self, robot: &RobotDescriptor) -> Vec<MeshBuffer>;
    fn draw_robot(&mut self, robot: &RobotDescriptor, pipeline: &wgpu::RenderPipeline, camera_buffer: &wgpu::Buffer, light_buffer: &wgpu::Buffer, transform_buffer: &wgpu::Buffer);
}

impl RobotGraphics for WGPUGraphics {
    fn robot_create_buffers(&mut self, robot: &RobotDescriptor) -> Vec<MeshBuffer> {
        robot.links.iter().map(|link| &link.visual.geometry).map(|mesh| self.create_mesh_buffer(mesh)).collect()
    }
    fn draw_robot(&mut self, robot: &RobotDescriptor, pipeline: &wgpu::RenderPipeline, camera_buffer: &wgpu::Buffer, light_buffer: &wgpu::Buffer, transform_buffer: &wgpu::Buffer) {
        let buffers = self.robot_create_buffers(robot);
        self.draw_mesh_list(pipeline, &buffers, camera_buffer, light_buffer, transform_buffer);
    }
}
