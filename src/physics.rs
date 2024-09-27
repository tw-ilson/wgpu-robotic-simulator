
pub trait PhysicsProgram {
    fn new() -> Self;
    fn setup(&mut self, scene: &str);
    fn step(&mut self);
    fn apply_forces(&mut self);
    fn update_kinematics(&mut self);
    fn detect_collisions(&mut self);
    fn solve_constraints(&mut self);
}

// pub struct PhysicsWorld {
//     objects: Vec<Polyhedron>
// }

#[derive(Debug, Copy, Clone)]
pub struct FreeBody {
    // linear kinematic info
    pub posn: glm::Vec3,
    pub vel: glm::Vec3,
    pub force: glm::Vec3,
    pub mass: f32,
    // acc: Vec2,

    //angular kinematic info
    pub theta: f32,
    pub omega: f32,
    // alpha: f32,
}

