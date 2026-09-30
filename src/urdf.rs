use crate::geometry::{BoxMesh, CylinderMesh, Polyhedron, SphereMesh, Transform, TriMesh};
use glm;
use std::str::FromStr;
use xml::attribute::OwnedAttribute;
use xml::reader::XmlEvent;
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
    pub origin: Origin,
    /// Accumulated world transform of the link frame. Recomputed by `build()`.
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
    pub origin: Origin,
    /// World transform of the visual. Recomputed by `build()` as
    /// `link_world * origin`; do not overwrite the local origin.
    pub transform: Transform,
    pub geometry: Polyhedron,
    pub material: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct CollisionBody {
    pub origin: Origin,
    /// World transform of the collision shape. Recomputed by `build()`.
    pub transform: Transform,
    pub geometry: Polyhedron,
}

#[derive(Default, Debug, Clone)]
pub struct Link {
    pub link_name: String,
    /// URDF allows several `<visual>` elements per link; all are kept.
    pub visuals: Vec<VisualBody>,
    pub inertial: InertialBody,
    /// URDF allows several `<collision>` elements per link; all are kept.
    pub collisions: Vec<CollisionBody>,
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
    origin: Origin,
    transform: Transform,
    /// Axis in the joint frame. The URDF spec defaults a missing <axis> to
    /// (1,0,0); the parser applies that, so this is only None for
    /// programmatically built joints.
    axis: Option<glm::Vec3>,
    limits: Option<JointLimits>,
    dynamics: Option<JointDynamics>,
}

#[derive(Default, Debug, Clone)]
pub struct RobotDescriptor {
    pub name: Option<String>,
    pub links: Vec<Link>,
    pub joints: Vec<Joint>,
}

type ParseRobotError = Box<dyn std::error::Error>;

fn xml_error<E: std::fmt::Display>(e: E) -> ParseRobotError {
    format!("XML error: {e}").into()
}

/// Look up an attribute by name. XML attribute order is not significant,
/// so never index into the attribute list positionally.
fn attr<'a>(attributes: &'a [OwnedAttribute], name: &str) -> Result<&'a str, ParseRobotError> {
    attributes
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.as_str())
        .ok_or_else(|| format!("expected attribute '{name}'").into())
}

/// Consume events until the matching end tag of the element whose start tag
/// was just read. Used to skip unsupported elements and their subtrees.
fn skip_element(xml_parser: &mut EventReader<&[u8]>) -> Result<(), ParseRobotError> {
    let mut depth = 1u32;
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement { .. } => depth += 1,
            XmlEvent::EndElement { .. } => {
                depth -= 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            _ => {}
        }
    }
}

//gets position, rotation from origin element
fn parse_3f(s: &str) -> Result<glm::Vec3, ParseRobotError> {
    let v: Vec<f32> = s
        .split_whitespace()
        .map(|ns| ns.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()
        .map_err(|e| format!("expected 3 floats in '{s}': {e}"))?;
    if v.len() != 3 {
        return Err(format!("expected 3 floats in '{s}', got {}", v.len()).into());
    }
    Ok(glm::vec3(v[0], v[1], v[2]))
}

fn parse_4f(s: &str) -> Result<glm::Vec4, ParseRobotError> {
    let v: Vec<f32> = s
        .split_whitespace()
        .map(|ns| ns.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()
        .map_err(|e| format!("expected 4 floats in '{s}': {e}"))?;
    if v.len() != 4 {
        return Err(format!("expected 4 floats in '{s}', got {}", v.len()).into());
    }
    Ok(glm::vec4(v[0], v[1], v[2], v[3]))
}

fn parse_origin(attributes: &[OwnedAttribute]) -> Result<Origin, ParseRobotError> {
    // Per the URDF spec both attributes are optional and default to "0 0 0";
    // real-world files (e.g. xacro output) often omit xyz.
    let xyz = attributes
        .iter()
        .find(|a| a.name.local_name == "xyz")
        .map(|a| parse_3f(&a.value))
        .transpose()?
        .unwrap_or_default();
    let rpy = attributes
        .iter()
        .find(|a| a.name.local_name == "rpy")
        .map(|a| parse_3f(&a.value))
        .transpose()?;
    Ok(Origin { xyz, rpy })
}

fn parse_link_geometry(xml_parser: &mut EventReader<&[u8]>) -> Result<Polyhedron, ParseRobotError> {
    let mut shape: Option<Polyhedron> = None;
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "mesh" => {
                    let fname = attr(&attributes, "filename")?.to_owned();
                    // Mesh files are external resources: a missing file or an
                    // unsupported format (e.g. .dae) must not abort the whole
                    // parse. Warn loudly and leave an empty geometry so the
                    // kinematic structure still comes through.
                    match Polyhedron::load_file(&fname) {
                        Ok(mut poly) => {
                            if let Some(scale) =
                                attributes.iter().find(|a| a.name.local_name == "scale")
                            {
                                poly.scale_xyz(parse_3f(&scale.value)?);
                            }
                            shape = Some(poly);
                        }
                        Err(e) => {
                            eprintln!("warning: {e}; using empty geometry");
                            shape = Some(Polyhedron::default());
                        }
                    }
                }
                "box" => {
                    let size = parse_3f(attr(&attributes, "size")?)?;
                    shape = Some(Polyhedron::from(TriMesh::create_box(size)));
                }
                "cylinder" => {
                    let l: f32 = attr(&attributes, "length")?
                        .parse()
                        .map_err(|e| format!("bad cylinder length: {e}"))?;
                    let r: f32 = attr(&attributes, "radius")?
                        .parse()
                        .map_err(|e| format!("bad cylinder radius: {e}"))?;
                    shape = Some(Polyhedron::from(TriMesh::create_cylinder(r, l, 30)));
                }
                "sphere" => {
                    let r: f32 = attr(&attributes, "radius")?
                        .parse()
                        .map_err(|e| format!("bad sphere radius: {e}"))?;
                    shape = Some(Polyhedron::from(TriMesh::create_sphere(r, 20, 20)));
                }
                other => {
                    // Unknown shape (e.g. <capsule> from newer URDF revisions):
                    // warn and continue with an empty geometry rather than
                    // failing the whole file.
                    eprintln!("warning: skipping unknown geometry '{other}'");
                    skip_element(xml_parser)?;
                    shape = Some(Polyhedron::default());
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "geometry" {
                    return shape.ok_or_else(|| "empty <geometry> element".into());
                }
            }
            _ => {}
        }
    }
}

fn parse_link_visual(
    xml_parser: &mut EventReader<&[u8]>,
    materials: &mut Vec<Material>,
) -> Result<VisualBody, ParseRobotError> {
    let mut visual = VisualBody::default();
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "origin" => {
                    visual.origin = parse_origin(&attributes)?;
                    visual.transform = visual.origin.into();
                }
                "geometry" => visual.geometry = parse_link_geometry(xml_parser)?,
                "material" => {
                    let mat_name = attr(&attributes, "name")?.to_owned();
                    visual.material = Some(mat_name.clone());
                    // Inline material definition: register it. A bare
                    // reference (no <color>) simply yields Err here, which
                    // is fine -- the top-level material pass resolves it.
                    if let Ok(mat) = parse_material(xml_parser, mat_name) {
                        materials.push(mat);
                    }
                }
                other => {
                    eprintln!("warning: skipping unknown visual element '{other}'");
                    skip_element(xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "visual" {
                    return Ok(visual);
                }
            }
            _ => {}
        }
    }
}

fn parse_link_collision(
    xml_parser: &mut EventReader<&[u8]>,
) -> Result<CollisionBody, ParseRobotError> {
    let mut collision = CollisionBody::default();
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "origin" => {
                    collision.origin = parse_origin(&attributes)?;
                    collision.transform = collision.origin.into();
                }
                "geometry" => {
                    collision.geometry = parse_link_geometry(xml_parser)?;
                }
                other => {
                    eprintln!("warning: skipping unknown collision element '{other}'");
                    skip_element(xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "collision" {
                    return Ok(collision);
                }
            }
            _ => {}
        }
    }
}

fn parse_link_inertial(
    xml_parser: &mut EventReader<&[u8]>,
) -> Result<InertialBody, ParseRobotError> {
    let mut origin = Origin::default();
    let mut mass: Option<f32> = None;
    let mut inertia: Option<[f32; 6]> = None;
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "origin" => origin = parse_origin(&attributes)?,
                "mass" => {
                    mass = Some(
                        attr(&attributes, "value")?
                            .parse()
                            .map_err(|e| format!("bad mass value: {e}"))?,
                    )
                }
                "inertia" => {
                    let mut vals = [0.0f32; 6];
                    for (slot, key) in vals.iter_mut().zip(["ixx", "iyy", "izz", "ixy", "ixz", "iyz"]) {
                        if let Some(a) = attributes.iter().find(|a| a.name.local_name == key) {
                            *slot = a
                                .value
                                .parse()
                                .map_err(|e| format!("bad inertia {key}: {e}"))?;
                        }
                    }
                    inertia = Some(vals);
                }
                other => {
                    eprintln!("warning: skipping unknown inertial element '{other}'");
                    skip_element(xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "inertial" {
                    let mass = mass.ok_or("inertial body requires mass!")?;
                    let [ixx, iyy, izz, ixy, ixz, iyz] =
                        inertia.ok_or("inertial body requires moments of inertia!")?;
                    return Ok(InertialBody {
                        origin,
                        transform: origin.into(),
                        mass,
                        ixx,
                        iyy,
                        izz,
                        ixy,
                        ixz,
                        iyz,
                    });
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
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "visual" => link.visuals.push(parse_link_visual(xml_parser, materials)?),
                "inertial" => link.inertial = parse_link_inertial(xml_parser)?,
                "collision" => link.collisions.push(parse_link_collision(xml_parser)?),
                other => {
                    eprintln!("warning: skipping unknown link element '{other}'");
                    skip_element(xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "link" {
                    return Ok(link);
                }
            }
            _ => {}
        }
    }
}

/// A joint whose parent/child link names have not been resolved to link
/// indices yet. URDF allows a joint to reference links defined later in the
/// file (or interleaved with joints), so resolution happens after the whole
/// document is parsed.
struct UnresolvedJoint {
    joint_name: String,
    joint_type: JointType,
    parent_name: String,
    child_name: String,
    origin: Origin,
    axis: Option<glm::Vec3>,
    limits: Option<JointLimits>,
    dynamics: Option<JointDynamics>,
}

pub fn parse_joint(
    xml_parser: &mut EventReader<&[u8]>,
    joint_name: String,
    joint_type: JointType,
) -> Result<UnresolvedJoint, ParseRobotError> {
    let mut parent_name: Option<String> = None;
    let mut child_name: Option<String> = None;
    let mut origin: Option<Origin> = None;
    let mut axis: Option<glm::Vec3> = None;
    let mut limits: Option<JointLimits> = None;
    let mut dynamics: Option<JointDynamics> = None;

    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "parent" => parent_name = Some(attr(&attributes, "link")?.to_owned()),
                "child" => child_name = Some(attr(&attributes, "link")?.to_owned()),
                "origin" => origin = Some(parse_origin(&attributes)?),
                "axis" => axis = Some(parse_3f(attr(&attributes, "xyz")?)?),
                "limit" => {
                    let mut lim = JointLimits::default();
                    for a in &attributes {
                        let v: f32 = a
                            .value
                            .parse()
                            .map_err(|e| format!("bad limit attribute: {e}"))?;
                        match a.name.local_name.as_str() {
                            "effort" => lim.effort = v,
                            "lower" => lim.lower = v,
                            "upper" => lim.upper = v,
                            "velocity" => lim.velocity = v,
                            // Ignore extensibility attributes instead of panicking.
                            _ => {}
                        }
                    }
                    limits = Some(lim);
                }
                "dynamics" => {
                    let mut dyn_ = JointDynamics::default();
                    for a in &attributes {
                        let v: f32 = a
                            .value
                            .parse()
                            .map_err(|e| format!("bad dynamics attribute: {e}"))?;
                        match a.name.local_name.as_str() {
                            "damping" => dyn_.damping = v,
                            "friction" => dyn_.friction = v,
                            _ => {}
                        }
                    }
                    dynamics = Some(dyn_);
                }
                // Elements like <calibration>, <safety_controller> and
                // <mimic> are valid URDF but irrelevant here; skip them
                // instead of failing the parse.
                _ => {
                    skip_element(xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "joint" {
                    break;
                }
            }
            _ => {}
        }
    }
    let p_name = parent_name.ok_or("parent element is required!")?;
    let c_name = child_name.ok_or("child element is required!")?;
    let origin = origin.unwrap_or_default();
    // <axis> is optional per the URDF spec and defaults to (1,0,0).
    let axis = Some(axis.unwrap_or_else(|| glm::vec3(1.0, 0.0, 0.0)));
    Ok(UnresolvedJoint {
        joint_name,
        joint_type,
        parent_name: p_name,
        child_name: c_name,
        origin,
        axis,
        limits,
        dynamics,
    })
}

#[derive(Debug, Default, PartialEq, Clone)]
struct Material {
    name: String,
    color: glm::Vec3,
}

fn parse_material(
    xml_parser: &mut EventReader<&[u8]>,
    material_name: String,
) -> Result<Material, ParseRobotError> {
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "color" => {
                    let rgba = parse_4f(attr(&attributes, "rgba")?)?;
                    return Ok(Material {
                        name: material_name,
                        color: rgba.xyz(),
                    });
                }
                // Textures aren't supported; skip the element instead of
                // panicking so the rest of the material still parses.
                "texture" => skip_element(xml_parser)?,
                other => {
                    return Err(format!("unknown element '{other}' in material").into());
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "material" {
                    return Err("could not parse material".into());
                }
            }
            _ => {}
        }
    }
}

fn parse_robot(
    mut xml_parser: EventReader<&[u8]>,
    robot_name: Option<String>,
) -> Result<RobotDescriptor, ParseRobotError> {
    let mut links = Vec::new();
    let mut joints = Vec::new();
    let mut pending_joints = Vec::<UnresolvedJoint>::new();
    let mut materials = Vec::<Material>::new();
    loop {
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => match name.local_name.as_str() {
                "link" => {
                    let link_name = attr(&attributes, "name")?.to_owned();
                    links.push(parse_link(&mut xml_parser, link_name, &mut materials)?);
                }
                "joint" => {
                    let joint_name = attr(&attributes, "name")?.to_owned();
                    // Look the type up by name: XML attribute order is not
                    // significant, so positional indexing is a bug.
                    let joint_type = match attr(&attributes, "type")? {
                        "fixed" => JointType::Fixed,
                        "revolute" => JointType::Revolute,
                        "continuous" => JointType::Continuous,
                        "prismatic" => JointType::Prismatic,
                        "floating" => JointType::Floating,
                        other => return Err(format!("unrecognized joint type '{other}'").into()),
                    };
                    // Link names are resolved after the parse: URDF does not
                    // require <link> elements to precede the <joint>s that
                    // reference them (links and joints are often interleaved).
                    pending_joints.push(parse_joint(&mut xml_parser, joint_name, joint_type)?);
                }
                // Top-level <material> definitions (name + <color>), resolved
                // against visual references after the parse. A bare reference
                // carries no color; ignore it like the visual parser does.
                "material" => {
                    let mat_name = attr(&attributes, "name")?.to_owned();
                    if let Ok(mat) = parse_material(&mut xml_parser, mat_name) {
                        materials.push(mat);
                    }
                }
                // Transmissions, sensors and simulator extensions (e.g.
                // gazebo tags) are valid URDF but out of scope here; skip
                // their subtrees instead of failing.
                _ => {
                    skip_element(&mut xml_parser)?;
                }
            },
            XmlEvent::EndElement { name } => {
                if name.local_name == "robot" {
                    break;
                }
            }
            _ => {}
        }
    }

    // Resolve joint link references now that every link is known.
    for uj in pending_joints {
        let parent = links
            .iter()
            .position(|l| l.link_name == uj.parent_name)
            .ok_or_else(|| {
                format!(
                    "joint '{}' references unknown parent link '{}'",
                    uj.joint_name, uj.parent_name
                )
            })?;
        let child = links
            .iter()
            .position(|l| l.link_name == uj.child_name)
            .ok_or_else(|| {
                format!(
                    "joint '{}' references unknown child link '{}'",
                    uj.joint_name, uj.child_name
                )
            })?;
        let transform = Transform::from(uj.origin);
        joints.push(Joint {
            joint_name: uj.joint_name,
            joint_type: uj.joint_type,
            parent,
            child,
            origin: uj.origin,
            transform,
            axis: uj.axis,
            limits: uj.limits,
            dynamics: uj.dynamics,
        });
    }

    //setup colors
    for mat in &materials {
        for link in links.iter_mut() {
            for visual in link.visuals.iter_mut() {
                if visual.material.as_deref() == Some(mat.name.as_str()) {
                    visual.geometry.set_color(mat.color);
                }
            }
        }
    }

    Ok(RobotDescriptor {
        name: robot_name,
        links,
        joints,
    })
}

impl FromStr for RobotDescriptor {
    type Err = ParseRobotError;
    fn from_str(s: &str) -> Result<RobotDescriptor, ParseRobotError> {
        let mut xml_parser = EventReader::from_str(s);
        let mut robot_name: Option<String> = None;
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartDocument { .. } => {}
            _ => return Err("Is this a valid XML URDF file?".into()),
        }
        match xml_parser.next().map_err(xml_error)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if name.local_name != "robot" {
                    return Err("expected robot element as first element".into());
                }
                if let Some(a) = attributes.iter().find(|a| a.name.local_name == "name") {
                    robot_name = Some(a.value.clone());
                }
                parse_robot(xml_parser, robot_name)
            }
            _ => Err("expected robot element as first element".into()),
        }
    }
}

impl RobotDescriptor {
    /// Every visual in the robot, in link order. Mesh buffers, transform
    /// buffers and draw calls all iterate this order, keeping them in
    /// lockstep (mesh i is drawn with transform i).
    pub fn visuals(&self) -> impl Iterator<Item = &VisualBody> {
        self.links.iter().flat_map(|l| l.visuals.iter())
    }

    /// Index of the root link: the link that is no joint's child.
    /// Falls back to the first link for degenerate input.
    fn root_link_index(&self) -> usize {
        (0..self.links.len())
            .find(|&i| !self.joints.iter().any(|j| j.child == i))
            .unwrap_or(0)
    }

    /// Structural sanity checks over the parsed descriptor, for vetting
    /// third-party URDF files before simulating them. Returns a list of
    /// human-readable problems; an empty list means the kinematic graph
    /// looks sound (single root, all links reachable, no cycles, joint
    /// references valid, every actuated joint has an axis).
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.links.is_empty() {
            problems.push("robot has no links".to_owned());
            return problems;
        }
        let mut seen = std::collections::HashSet::new();
        for l in &self.links {
            if !seen.insert(l.link_name.as_str()) {
                problems.push(format!("duplicate link name '{}'", l.link_name));
            }
        }
        let mut seen_j = std::collections::HashSet::new();
        for j in &self.joints {
            if !seen_j.insert(j.joint_name.as_str()) {
                problems.push(format!("duplicate joint name '{}'", j.joint_name));
            }
            if j.parent >= self.links.len() {
                problems.push(format!(
                    "joint '{}' has parent index {} out of range",
                    j.joint_name, j.parent
                ));
            }
            if j.child >= self.links.len() {
                problems.push(format!(
                    "joint '{}' has child index {} out of range",
                    j.joint_name, j.child
                ));
            }
            if j.parent == j.child {
                problems.push(format!("joint '{}' connects a link to itself", j.joint_name));
            }
            match j.joint_type {
                JointType::Revolute | JointType::Prismatic | JointType::Continuous => {
                    if j.axis.is_none() {
                        problems.push(format!(
                            "joint '{}' ({:?}) has no <axis>; set_joint_position would panic",
                            j.joint_name, j.joint_type
                        ));
                    }
                }
                _ => {}
            }
            if let Some(lim) = j.limits {
                if lim.lower > lim.upper {
                    problems.push(format!(
                        "joint '{}' has inverted limits (lower {} > upper {})",
                        j.joint_name, lim.lower, lim.upper
                    ));
                }
            }
        }

        let is_child =
            |i: usize| -> bool { self.joints.iter().any(|j| j.child == i) };
        let roots: Vec<usize> = (0..self.links.len()).filter(|&i| !is_child(i)).collect();
        if roots.is_empty() {
            problems.push("no root link: every link is a joint child (cycle?)".to_owned());
        } else if roots.len() > 1 {
            let names: Vec<&str> =
                roots.iter().map(|&i| self.links[i].link_name.as_str()).collect();
            problems.push(format!(
                "{} root links, expected 1 (disconnected?): {}",
                roots.len(),
                names.join(", ")
            ));
        }

        // Reachability from the roots + cycle detection (iterative DFS with
        // 0 = unvisited, 1 = on stack, 2 = done).
        let mut color = vec![0u8; self.links.len()];
        let mut has_cycle = false;
        for &r in &roots {
            let mut stack = vec![(r, false)];
            while let Some((n, exiting)) = stack.pop() {
                if exiting {
                    color[n] = 2;
                    continue;
                }
                if color[n] == 2 {
                    continue;
                }
                if color[n] == 1 {
                    has_cycle = true;
                    continue;
                }
                color[n] = 1;
                stack.push((n, true));
                for c in self.joints.iter().filter(|j| j.parent == n).map(|j| j.child) {
                    if c < self.links.len() {
                        stack.push((c, false));
                    }
                }
            }
        }
        if has_cycle {
            problems.push("kinematic cycle detected".to_owned());
        }
        for (i, l) in self.links.iter().enumerate() {
            if color[i] == 0 {
                problems.push(format!("link '{}' is unreachable from the root", l.link_name));
            }
        }
        problems
    }

    /// Set joint positions. `theta` holds one value per joint, *including*
    /// fixed joints (their entries are ignored). Revolute and prismatic
    /// joints are clamped to their `<limit>` range when one is specified.
    pub fn set_joint_position(&mut self, theta: &[f32], relative: bool) {
        if theta.len() != self.joints.len() {
            panic!("expected {} got {}", self.joints.len(), theta.len())
        }
        for (th, j) in std::iter::zip(theta.iter(), self.joints.iter_mut()) {
            if !relative {
                j.transform = j.origin.into();
            }
            // Clamp to joint limits when present. (A missing bound leaves
            // the default 0.0, so only clamp on a sane range.)
            let limits = j.limits;
            let clamped = |t: f32| {
                if let Some(lim) = limits {
                    if lim.lower <= lim.upper {
                        return t.clamp(lim.lower, lim.upper);
                    }
                }
                t
            };
            match j.joint_type {
                JointType::Revolute => {
                    let axis = j.axis.expect("revolute joint requires axis!");
                    j.transform.rotate(axis, clamped(*th));
                }
                JointType::Prismatic => {
                    let axis = j.axis.expect("prismatic joint requires axis");
                    j.transform.translate(clamped(*th) * axis);
                }
                JointType::Continuous => {
                    let axis = j.axis.expect("continuous joint requires axis!");
                    j.transform.rotate(axis, *th);
                }
                JointType::Floating => { /* 6 DOF; not modeled */ }
                JointType::Fixed => { /* do nothing */ }
            }
        }
    }

    pub fn reset_joint_transforms(&mut self) {
        self.links.iter_mut().for_each(|l| {
            l.inertial.transform = l.inertial.origin.into();
            l.visuals
                .iter_mut()
                .for_each(|v| v.transform = v.origin.into());
            l.collisions
                .iter_mut()
                .for_each(|c| c.transform = c.origin.into());
        })
    }

    /// Forward kinematics: walk the joint tree from the root link and
    /// accumulate world transforms. Each visual/collision keeps its own
    /// `<origin>` offset relative to its link frame.
    pub fn build(&mut self) {
        self.reset_joint_transforms();
        if self.links.is_empty() {
            return;
        }
        let root = self.root_link_index().min(self.links.len() - 1);
        let root_world: Transform = self.links[root].inertial.origin.into();
        let mut stack = vec![(root, root_world)];
        while let Some((li, link_world)) = stack.pop() {
            // Child link worlds, computed from immutable borrows first.
            let children: Vec<(usize, Transform)> = self
                .joints
                .iter()
                .filter(|j| j.parent == li)
                .map(|j| {
                    let child_origin: Transform = self.links[j.child].inertial.origin.into();
                    (j.child, link_world * j.transform * child_origin)
                })
                .collect();
            let link = &mut self.links[li];
            link.inertial.transform = link_world;
            for v in link.visuals.iter_mut() {
                v.transform = link_world * Transform::from(v.origin);
            }
            for c in link.collisions.iter_mut() {
                c.transform = link_world * Transform::from(c.origin);
            }
            stack.extend(children);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translation(t: Transform) -> (f32, f32, f32) {
        (t.tmatrix[(0, 3)], t.tmatrix[(1, 3)], t.tmatrix[(2, 3)])
    }

    const SIMPLE: &str = r#"<?xml version="1.0"?>
<robot name="test">
  <link name="base">
    <visual><origin xyz="0 0 0"/><geometry><box size="1 1 1"/></geometry></visual>
    <visual><origin xyz="1 0 0"/><geometry><box size="1 1 1"/></geometry></visual>
    <inertial><origin xyz="0 0 0"/><mass value="1"/>
      <inertia ixx="1" iyy="1" izz="1" ixy="0" ixz="0" iyz="0"/></inertial>
  </link>
  <link name="child">
    <visual><origin xyz="0 2 0"/><geometry><sphere radius="0.5"/></geometry></visual>
    <inertial><origin xyz="0 0 0"/><mass value="1"/>
      <inertia ixx="1" iyy="1" izz="1" ixy="0" ixz="0" iyz="0"/></inertial>
  </link>
  <joint type="revolute" name="j1">
    <parent link="base"/>
    <child link="child"/>
    <origin xyz="0 0 5"/>
    <axis xyz="0 0 1"/>
    <limit lower="-1" upper="1" effort="10" velocity="10"/>
    <calibration rising="0" falling="0"/>
    <safety_controller k_velocity="1"/>
    <mimic joint="j0" multiplier="1" offset="0"/>
  </joint>
  <transmission name="t1"><type>SimpleTransmission</type>
    <joint name="j1"/><actuator name="m1"/></transmission>
  <sensor name="s1"/>
  <gazebo reference="base"><material>Gazebo/Blue</material></gazebo>
</robot>"#;

    #[test]
    fn parses_despite_attribute_order_and_unsupported_elements() {
        // type-before-name, rpy-before-xyz handled; transmission, sensor,
        // calibration, safety_controller, mimic and gazebo skipped.
        let robot = RobotDescriptor::from_str(SIMPLE).expect("should parse");
        assert_eq!(robot.links.len(), 2);
        assert_eq!(robot.joints.len(), 1);
        assert!(matches!(robot.joints[0].joint_type, JointType::Revolute));
    }

    #[test]
    fn keeps_all_visuals_per_link() {
        let robot = RobotDescriptor::from_str(SIMPLE).expect("should parse");
        let base = robot.links.iter().find(|l| l.link_name == "base").unwrap();
        assert_eq!(base.visuals.len(), 2, "both <visual> elements must be kept");
        assert_eq!(robot.visuals().count(), 3);
    }

    #[test]
    fn visual_origins_survive_fk() {
        let mut robot = RobotDescriptor::from_str(SIMPLE).expect("should parse");
        robot.set_joint_position(&[0.0], false);
        robot.build();
        let base = robot.links.iter().find(|l| l.link_name == "base").unwrap();
        let t0 = translation(base.visuals[0].transform);
        let t1 = translation(base.visuals[1].transform);
        assert!((t0.0 - 0.0).abs() < 1e-5 && (t0.1 - 0.0).abs() < 1e-5);
        assert!((t1.0 - 1.0).abs() < 1e-5, "visual origin must offset the visual, got {t1:?}");
        // child visual: joint origin (0,0,5) + visual origin (0,2,0)
        let child = robot.links.iter().find(|l| l.link_name == "child").unwrap();
        let tc = translation(child.visuals[0].transform);
        assert!((tc.0 - 0.0).abs() < 1e-5 && (tc.1 - 2.0).abs() < 1e-5 && (tc.2 - 5.0).abs() < 1e-5,
            "child visual world transform wrong: {tc:?}");
    }

    #[test]
    fn joint_limits_are_clamped() {
        let mut robot = RobotDescriptor::from_str(SIMPLE).expect("should parse");
        robot.set_joint_position(&[100.0], false);
        robot.build();
        let child = robot.links.iter().find(|l| l.link_name == "child").unwrap();
        // theta=100 clamped to upper=1 rad about z: visual at (0,2,0)
        // rotated by 1 rad -> (-2 sin1, 2 cos1, 5)
        let tc = translation(child.visuals[0].transform);
        let (ex, ey) = (-2.0 * 1f32.sin(), 2.0 * 1f32.cos());
        assert!((tc.0 - ex).abs() < 1e-4 && (tc.1 - ey).abs() < 1e-4,
            "expected clamped rotation, got {tc:?}");
    }

    #[test]
    fn root_link_found_when_not_first() {
        let urdf = r#"<?xml version="1.0"?>
<robot name="order">
  <link name="child">
    <visual><geometry><box size="1 1 1"/></geometry></visual>
    <inertial><mass value="1"/>
      <inertia ixx="1" iyy="1" izz="1" ixy="0" ixz="0" iyz="0"/></inertial>
  </link>
  <link name="base">
    <visual><geometry><box size="1 1 1"/></geometry></visual>
    <inertial><mass value="1"/>
      <inertia ixx="1" iyy="1" izz="1" ixy="0" ixz="0" iyz="0"/></inertial>
  </link>
  <joint name="j" type="fixed"><parent link="base"/><child link="child"/>
    <origin xyz="0 0 3"/></joint>
</robot>"#;
        let mut robot = RobotDescriptor::from_str(urdf).expect("should parse");
        robot.build();
        let child = robot.links.iter().find(|l| l.link_name == "child").unwrap();
        let tc = translation(child.visuals[0].transform);
        assert!((tc.2 - 3.0).abs() < 1e-5, "child should sit at z=3, got {tc:?}");
    }

    #[test]
    fn malformed_floats_are_errors_not_panics() {
        let urdf = r#"<?xml version="1.0"?>
<robot name="bad">
  <link name="base">
    <visual><origin xyz="0 0 banana"/><geometry><box size="1 1 1"/></geometry></visual>
    <inertial><mass value="1"/>
      <inertia ixx="1" iyy="1" izz="1" ixy="0" ixz="0" iyz="0"/></inertial>
  </link>
</robot>"#;
        assert!(RobotDescriptor::from_str(urdf).is_err());
    }

    #[test]
    fn xarm_parses_with_every_visual() {
        let robot = RobotDescriptor::from_str(include_str!("../assets/xarm.urdf"))
            .expect("xarm.urdf should parse");
        let total_visuals: usize = robot.links.iter().map(|l| l.visuals.len()).sum();
        assert!(robot.links.iter().any(|l| l.visuals.len() > 1),
            "xarm has links with several visuals");
        let base = robot.links.iter().find(|l| l.link_name == "base_link").unwrap();
        assert_eq!(base.visuals.len(), 2);
        // the empty "world" link contributes no visual (and no empty mesh)
        let world = robot.links.iter().find(|l| l.link_name == "world").unwrap();
        assert!(world.visuals.is_empty());
        assert_eq!(total_visuals, 12);
        // colors applied by name
        assert!(base.visuals.iter().all(|v| v.material.as_deref() == Some("blue")));
    }
}
