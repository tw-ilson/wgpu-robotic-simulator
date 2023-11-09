use bytemuck::{Pod, Zeroable};
use glm::Mat4;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

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
    eye_posn: glm::Vec3,
    view_direction: glm::Vec3,
    up_vector: glm::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraController {
    speed: f32,
    mouse_pressed:bool,
    old_mouse_posn: glm::Vec2,
    input_state: [bool; 4],
}
impl Default for CameraController {
    fn default() -> Self{
        Self {
            mouse_pressed:false,
            old_mouse_posn: glm::vec2(0.0, 0.0),
            speed: 0.05,
            input_state: [false, false, false, false],
        }
    }
}
impl CameraController {
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => {
                // self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
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
                    _ => false,
                }
            }
            _ => false,
        }
    }
    pub fn mouse_look(&mut self, cam: &mut Camera, delta_x: f32, delta_y: f32) {
        // let new_mouse_posn: glm::Vec2 = glm::vec2(mouse_x, mouse_y);
        // let delta: glm::Vec2 = self.old_mouse_posn - new_mouse_posn;
        let sensitivity = 0.1;
        let right_vec = glm::vec3(
                cam.view_direction.z,
                cam.view_direction.y,
                -cam.view_direction.x,
            );
        cam.view_direction = glm::rotate_vec3(&cam.view_direction, -delta_x * sensitivity, &cam.up_vector);
        cam.view_direction = glm::rotate_vec3(&cam.view_direction, delta_y * sensitivity, &right_vec);
        // self.old_mouse_posn = new_mouse_posn
    }
    pub fn update(&mut self, cam:&mut Camera) {
        if self.input_state[0] {
            self.move_forward(cam)
        }
        if self.input_state[1] {
            self.move_left(cam)
        }
        if self.input_state[2] {
            self.move_backward(cam)
        }
        if self.input_state[3] {
            self.move_right(cam)
        }
    }
    fn move_forward(&mut self, cam:&mut Camera) {
        cam.eye_posn += self.speed * cam.view_direction;
    }
    fn move_backward(&mut self, cam:&mut Camera) {
        cam.eye_posn -= self.speed * cam.view_direction;
    }
    fn move_left(&mut self, cam:&mut Camera) {
        cam.eye_posn += self.speed
            * glm::vec3(
                cam.view_direction.z,
                cam.view_direction.y,
                -cam.view_direction.x,
            );
    }
    fn move_right(&mut self, cam:&mut Camera ) {
        cam.eye_posn -= self.speed
            * glm::vec3(
                cam.view_direction.z,
                cam.view_direction.y,
                -cam.view_direction.x,
            );
    }
    fn move_up(&mut self, cam:&mut Camera) {
        cam.eye_posn.y += self.speed;
    }
    fn move_down(&mut self, cam:&mut Camera) {
        cam.eye_posn.y -= self.speed;
    }

}

impl Camera {
    pub fn new(w: u32, h: u32) -> Self {
        println!("Created a Camera");
        Self {
            eye_posn: glm::vec3(0.0, 0.0, 2.0),
            view_direction: glm::vec3(0.0, 0.0, -1.0),
            up_vector: glm::vec3(0.0, 1.0, 0.0),
            aspect: (w as f32) / (h as f32),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
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
}
