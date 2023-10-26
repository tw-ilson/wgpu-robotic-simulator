use bytemuck::{Pod, Zeroable};
use glm::Mat4;
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

// changes from OPENGL coordinate system to DIRECTX coordinate system
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glm::Mat4 = glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view_proj: Mat4,
}
unsafe impl Zeroable for CameraUniform {}
unsafe impl Pod for CameraUniform {}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::identity(),
        }
    }
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.get_view_projection_matrix()
    }
}
// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// struct InputState {
//     w:bool, a:bool, s:bool, d:bool
// }
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    old_mouse_posn: glm::Vec2,
    eye_posn: glm::Vec3,
    view_direction: glm::Vec3,
    up_vector: glm::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    speed: f32,
    input_state: [bool;4],
}

impl Camera {
    pub fn new(w: u32, h: u32) -> Self {
        println!("Created a Camera");
        Self {
            old_mouse_posn: glm::vec2(0.0, 0.0),
            eye_posn: glm::vec3(0.0, 0.0, 2.0),
            view_direction: glm::vec3(0.0, 0.0, -1.0),
            up_vector: glm::vec3(0.0, 1.0, 0.0),
            aspect: (w as f32) / (h as f32),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            speed: 0.05,
            input_state: [false, false, false, false ]
        }
    }
    pub fn get_view_projection_matrix(&self) -> glm::Mat4 {
        let view = glm::look_at(
            &self.eye_posn,
            &(self.eye_posn + self.view_direction),
            &self.up_vector,
        );
        let proj = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
    pub fn mouse_look(&mut self, mouse_x: isize, mouse_y: isize) {}
    pub fn move_forward(&mut self) {
        self.eye_posn += self.speed * self.view_direction;
    }
    pub fn move_backward(&mut self) {
        self.eye_posn -= self.speed * self.view_direction;
    }
    pub fn move_left(&mut self) {
        self.eye_posn += self.speed
            * glm::vec3(
                self.view_direction.z,
                self.view_direction.y,
                -self.view_direction.x,
            );
    }
    pub fn move_right(&mut self) {
        self.eye_posn -= self.speed
            * glm::vec3(
                self.view_direction.z,
                self.view_direction.y,
                -self.view_direction.x,
            );
    }
    pub fn move_up(&mut self) {
        self.eye_posn.y += self.speed;
    }
    pub fn move_down(&mut self) {
        self.eye_posn.y -= self.speed;
    }
    pub fn set_eye_posn(&mut self, x: f32, y: f32, z: f32) {
        self.eye_posn.x = x;
        self.eye_posn.y = y;
        self.eye_posn.z = z
    }
    pub fn get_eye_posn(&self) -> glm::Vec3 {
        self.eye_posn
    }
    pub fn get_view_direction(&self) -> glm::Vec3 {
        self.view_direction
    }
    pub fn update(&mut self) {
        if self.input_state[0] {
            self.move_forward()
        }
        if self.input_state[1] {
            self.move_left()
        }
        if self.input_state[2] {
            self.move_backward()
        }
        if self.input_state[3] {
            self.move_right()
        }
        // self.input_state = [false,false,false,false]
    }
    pub fn process_events(&mut self, event: &WindowEvent) -> bool{
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.input_state[0] = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.input_state[1] = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.input_state[2] = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.input_state[3] = is_pressed;
                        true
                    }
                    _ => false
                }
            }
            _ => false,
        }
    }
}
