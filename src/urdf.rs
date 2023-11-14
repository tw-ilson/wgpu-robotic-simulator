use crate::geometry::Polyhedron;
use glm;

enum JointType {
    Rotational,
    Prismatic,
}
struct Link {
    origin: glm::Vec3,
    rpy: glm::Vec3,
    geometry: Polyhedron
}
struct Joint {
    origin: glm::Vec3,
    rpy: glm::Vec3,
    axis: glm::Vec3,
    parent: Link, 
    child: Option<Link>,
    upper_limit: f32,
    lower_limit: f32,
    velocity_limit: f32,

}
struct RobotDescriptor {
    links: Vec<Polyhedron>
}
