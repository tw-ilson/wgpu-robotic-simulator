use crate::geometry::{BoxMesh, CylinderMesh, Polyhedron, TriMesh, Transform};
use crate::wgpu_program::WGPUGraphics;
use glm;
use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;
use xml::reader::{XmlEvent, XmlEvent::*};
use xml::EventReader;

#[derive(Default, Debug, Copy, Clone)]
pub struct Origin {
    xyz: glm::Vec3,
    rpy: Option<glm::Vec3>,
}

#[derive(Default, Debug, Clone)]
pub struct Link {
    pub link_name: String,
    pub geometry: Polyhedron,
    // pub origin: Origin,
    // pub color: Option<glm::Vec3>,
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
    effort: Option<f32>,
    lower: Option<f32>,
    upper: Option<f32>,
    velocity: Option<f32>,
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
        .collect::<Result<Vec<f32>, _>>()?
        .try_into()
        .unwrap();
    Ok(v.into())
}
fn parse_4f(s: &str) -> Result<glm::Vec4, ParseRobotError> {
    let v: [f32; 4] = s
        .split_whitespace()
        .map(|ns| ns.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()?
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
        // attributes.iter().inspect(|a| println!("{} -- {}", a.name.local_name, a.value)).collect_vec();
        let xyz_attr = attributes.iter().find(|&a| a.name.local_name == "xyz").ok_or("expected attribute xyz")?;
        let xyz = parse_3f(&xyz_attr.value)?;
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


fn create_sphere() -> Polyhedron {
    unimplemented!()
}

fn parse_link_geometry(
    xml_parser: &mut EventReader<&[u8]>,
    mut link: Link,
) -> Result<Link, ParseRobotError> {
    loop {
        let event = xml_parser.next();
        match event? {
            StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "mesh" => {
                    let fname = &attributes
                        .get(0)
                        .ok_or("expected file name attribute")?
                        .value;
                    let fstring = std::fs::read_to_string(fname)?;
                    link.geometry = Polyhedron::from(fstring);
                }
                "box" | "cylinder" | "sphere" => match name.local_name.as_str() {
                    "box" => {
                        let size_attr = attributes.get(0).ok_or("expected sized")?;
                        if size_attr.name.local_name == "size" {
                            let size = parse_3f(&size_attr.value)?;
                            link.geometry = TriMesh::create_box(size).into();
                        } else {
                            return Err("box requires size attribute".into())
                        }
                    }
                    "cylinder" => {
                        let l = attributes
                            .iter()
                            .find(|&a| a.name.local_name == "length")
                            .ok_or("cylinder requires length")?
                            .value
                            .parse::<f32>()?;
                        let r = attributes
                            .iter()
                            .find(|&a| a.name.local_name == "radius")
                            .ok_or("cylinder requires radius")?
                            .value
                            .parse::<f32>()?;
                        link.geometry = TriMesh::create_cylinder(r, l, 30).into();
                    }
                    "sphere" => {
                        link.geometry = create_sphere();
                    }
                    _ => return Err("unknown element".into()),
                },
                _ => {}
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
    materials: &Vec<Material>,
) -> Result<Link, ParseRobotError> {
    let mut transform: Option<Transform> = None;
    loop {
        let event = xml_parser.next();
        match event.clone()? {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                "origin" => {
                    let Origin{xyz, rpy} = parse_origin(event?)?;
                    // println!("{} -- xyz: {}, rpy {}", link.link_name, xyz, rpy.unwrap_or_default());
                    transform = Some(Transform::new(xyz, rpy.unwrap_or_default()));
                    },
                "geometry" => link = parse_link_geometry(xml_parser, link)?,
                "material" => {
                    let mat_name = &attributes
                        .iter()
                        .find(|&a| a.name.local_name == "name")
                        .ok_or("material requires name")?
                        .value;
                    link.geometry.set_color(materials.iter().find(|mat| mat.name == *mat_name).ok_or("unkown material!")?.color);
                    link.geometry.transform = transform.unwrap_or_default();
                },
                _ => {}
            },
            EndElement { name } => {
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
    materials: &Vec<Material>,
) -> Result<Link, ParseRobotError> {
    loop {
        let event = xml_parser.next();
        match event.clone()? {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                // "origin" => {
                //     let Origin{xyz, rpy} = parse_origin(event?)?;
                //     link.geometry.transform = Transform::new(xyz, rpy.unwrap_or_default());
                //     link.geometry.update_base();
                //     },
                // "geometry" => link = parse_link_geometry(xml_parser, link)?,
                // "material" => {
                //     let mat_name = &attributes
                //         .iter()
                //         .find(|&a| a.name.local_name == "name")
                //         .ok_or("material requires name")?
                //         .value;
                //     link.geometry.set_color(materials.iter().find(|mat| mat.name == *mat_name).ok_or("unkown material!")?.color);
                // },
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
    materials: &Vec<Material>,
) -> Result<Link, ParseRobotError> {
    loop {
        let event = xml_parser.next();
        match event.clone()? {
            StartElement { name, attributes, .. } => match name.local_name.as_str() {
                // "origin" => {
                //     let Origin{xyz, rpy} = parse_origin(event?)?;
                //     link.geometry.transform = Transform::new(xyz, rpy.unwrap_or_default());
                //     link.geometry.update_base();
                //     },
                // "geometry" => link = parse_link_geometry(xml_parser, link)?,
                // "material" => {
                //     let mat_name = &attributes
                //         .iter()
                //         .find(|&a| a.name.local_name == "name")
                //         .ok_or("material requires name")?
                //         .value;
                //     link.geometry.set_color(materials.iter().find(|mat| mat.name == *mat_name).ok_or("unkown material!")?.color);
                // },
                _ => {}
            },
            EndElement { name } => {
                // println!("{}", name.local_name);
                if name.local_name == "inertial" {
                    return Ok(link);
                }
            }
            _ => {}
        }
    }
}

fn parse_link(
    xml_parser: &mut EventReader<&[u8]>,
    link_name: String,
    materials: &Vec<Material>,
) -> Result<Link, ParseRobotError> {
    let mut link = Link::default();
    link.link_name = link_name;
    loop {
        let event = xml_parser.next();
        match event.clone()? {
            StartElement {
                name, ..
            } => {
                match name.local_name.as_str() {
                    "visual" => {
                        link = parse_link_visual(xml_parser, link, materials)?;
                    }
                    "inertial" => {link = parse_link_inertial(xml_parser, link, materials)?},
                    "collision" => {link = parse_link_collision(xml_parser, link, materials)?},
                    _ => {}
                }
            }
            EndElement { name } => {
                if name.local_name == "link" {
                    return Ok(link);
                }
            }
            _ => {} // println!("{:#?}",event.clone());
                    // panic!()}
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

    loop {
        let event = xml_parser.next();
        match event.clone()? {
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
                "origin" => origin = Some(parse_origin(event.clone()?)?),
                "axis" => {
                    let attr = attributes.get(0).ok_or("expected attribute xyz")?;
                    axis = Some(parse_3f(&attr.value)?);
                }
                "limit" => {
                    let (mut effort, mut lower, mut upper, mut velocity) = (None, None, None, None);
                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            "effort" => effort = Some(attr.value.parse::<f32>()?),
                            "lower" => lower = Some(attr.value.parse::<f32>()?),
                            "upper" => upper = Some(attr.value.parse::<f32>()?),
                            "velocity" => velocity = Some(attr.value.parse::<f32>()?),
                            _ => {}
                        }
                    }
                    limits = Some(JointLimits {
                        effort,
                        lower,
                        upper,
                        velocity,
                    });
                }
                _ => {
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
                // panic!("{:?}", event.clone()?)
            }
        }
    }
    let (parent, child);
    let p_name = parent_name.ok_or("parent element is required!")?;
    parent = links
        .iter()
        .position(|l| l.link_name == p_name)
        .ok_or(format!("no known link with name {}", p_name))?;
    let c_name = child_name.ok_or("child element is required!")?;
    child = links
        .iter()
        .position(|l| l.link_name == c_name)
        .ok_or(format!("no known link with name {}", c_name))?;
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
    })
}

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
                        let attr = attributes.iter().find(|&a| a.name.local_name == "rgba").ok_or("color must have rgba value")?;
                        color = parse_4f(&attr.value)?.xyz();
                        // println!("color: {}", color);
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
        match event? {
            StartElement {
                name, attributes, ..
            } => {
                let attr = attributes.get(0).ok_or("link requires name")?;
                assert!(attr.name.local_name == "name");
                attr_name = attr.value.to_owned();

                match name.local_name.as_str() {
                    "link" => links.push(parse_link(&mut xml_parser, attr_name, &materials)?),
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
                        joints.push(parse_joint(&mut xml_parser, attr_name, joint_type, &links)?)
                    }
                    "material" => materials.push(parse_material(&mut xml_parser, attr_name)?),
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
        match event? {
            StartDocument { .. } => {}
            _ => return Err("Is this a valid XML URDF file?".into()),
        }
        let event = xml_parser.next();
        match event? {
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
                // println!("{}\n{}",cur_link.link_name, cur_link.geometry.transform);
                // println!("J\n{}",j.transform);
                let tf = cur_link.geometry.transform * j.transform * self.links[j.child].geometry.transform;
                println!("{}\n{}\n{}",self.links[j.child].link_name, self.links[j.child].geometry.transform, tf);
                (j.child, tf)
            })
            // .inspect(|(_, t)| println!("{:?}", t))
            .collect_vec()
    }
    // Walk the DAG
    pub fn build(&mut self) {
        let base_link = self.links.get(0).expect("No links found?");
        let mut child_transforms = self.walk_children(base_link);
        loop {
            let mut queue: Option<Vec<(usize, Transform)>> = None;
            for (c_id, c_tf) in &child_transforms {
                // println!("{}\n{}", self.links[*c_id].link_name, c_tf);
                self.links[*c_id].geometry.transform = *c_tf;
                queue = Some(self.walk_children(&self.links[*c_id]));
            }
            if let Some(qlist) = queue {
                child_transforms = qlist;
            } else {
                break;
            }
        }
        self.links.iter_mut()
        // .inspect(|l| println!("{}", l.geometry.transform))
        .for_each(|l| l.geometry.update_base())
    }
}

pub trait RobotGraphics {
    fn robot_create_buffers(&self, robot: &RobotDescriptor) -> wgpu::Buffer;
    fn draw_robot(&self, robot: &RobotDescriptor);
}

// impl RobotGraphics for WGPUGraphics {
//     fn robot_create_buffers(&self, _robot: &RobotDescriptor) -> wgpu::Buffer {
//         unimplemented!()
//     }
//     fn draw_robot(&self, robot: &RobotDescriptor, pipeline: &wgpu::RenderPipeline, camera_buffer:&wgpu::Buffer, light_buffer: &wgpu::Buffer, transform_buffer: &wgpu::Buffer) {
//         self.draw_mesh_list(pipeline, buffer_list, mesh_list, camera_buffer, light_buffer, transform_buffer)
//     }
// }
