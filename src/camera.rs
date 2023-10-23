
struct Camera {
    old_mouse_posn: glm::Vec2,
    eye_posn: glm::Vec3,
    view_direction: glm::Vec3,
    up_vector: glm::Vec3,
    aspect:f32, 
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn get_view_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at(
            &self.eye_posn,
            &(self.eye_posn + self.view_direction),
            &self.up_vector,
        );
        let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
        proj * view
    }
    fn new(w: u32, h: u32) -> Self {
        println!("Created a Camera");
        Self {
            old_mouse_posn: glm::vec2(0.0, 0.0),
            eye_posn: glm::vec3(0.0, 0.0, 0.0),
            view_direction: glm::vec3(0.0, 0.0, -1.0),
            up_vector: glm::vec3(0.0, 1.0, 0.0),
            aspect: (w as f32)/(h as f32),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        }
    }
    fn mouse_look(&mut self, mouse_x: isize, mouse_y: isize) {}
    fn move_forward(&mut self, speed: f32) {
        self.eye_posn += speed * self.view_direction;
    }
    fn move_backward(&mut self, speed: f32) {
        self.eye_posn -= speed * self.view_direction;
    }
    fn move_left(&mut self, speed: f32) {
        self.eye_posn += speed * glm::vec3(self.view_direction.z, self.view_direction.y, -self.view_direction.x);
    }
    fn move_right(&mut self, speed: f32) {
        self.eye_posn -= speed * glm::vec3(self.view_direction.z, self.view_direction.y, -self.view_direction.x);
    }
    fn move_up(&mut self, speed: f32) {
        self.eye_posn.y += speed;
    }
    fn move_down(&mut self, speed: f32) {
        self.eye_posn.y -= speed;
    }
    fn set_eye_posn(&mut self, x: f32, y: f32, z: f32) {
        self.eye_posn.x = x;
        self.eye_posn.y = y;
        self.eye_posn.z = z
    }
    fn get_eye_posn(&self) -> glm::Vec3 {
        self.eye_posn
    }
    fn get_view_direction(&self) -> glm::Vec3 {
        self.view_direction
    }
}
