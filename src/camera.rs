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
}
#[repr(C)]
#[derive(Debug)]
pub struct Camera {
    eye_posn: glm::Vec3,
    view_direction: glm::Vec3,
    up_vector: glm::Vec3,
    aspect: f32,
    fov: f32,
    near: f32,
    far: f32,
}
impl Camera {
    pub fn new(w: u32, h: u32) -> Self {
        // println!("Created a Camera");
        let new_camera = Self {
            eye_posn: glm::vec3(0.0, -1.0, 0.0),
            view_direction: glm::vec3(0.0, 1.0, 0.0),
            up_vector: glm::vec3(0.0, 0.0, 1.0),
            aspect: (w as f32) / (h as f32),
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };
        new_camera
    }
    pub fn update_view_proj(&mut self, camera_uniform: &mut CameraUniform) {
        camera_uniform.view_proj = self.get_view_projection_matrix();
    }
    fn get_view_projection_matrix(&mut self) -> glm::Mat4 {
        let view = glm::look_at(
            &self.eye_posn,
            // &(self.eye_posn + self.view_direction),
            &glm::vec3(0., 0., self.eye_posn.z),
            &self.up_vector,
        );
        self.view_direction = glm::vec3(-self.eye_posn.x, -self.eye_posn.y, -self.eye_posn.z);
        let proj = glm::perspective(self.aspect, self.fov, self.near, self.far);
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct InputState {
    left: bool,
    right: bool,
    forward: bool,
    backward: bool,
    up: bool,
    down: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraController {
    speed: f32,
    mouse_pressed: bool,
    old_mouse_posn: glm::Vec2,
    input_state: InputState,
}
impl Default for CameraController {
    fn default() -> Self {
        Self {
            mouse_pressed: false,
            old_mouse_posn: glm::vec2(0.0, 0.0),
            speed: 0.05,
            input_state: InputState {
                left: false,
                right: false,
                forward: false,
                backward: false,
                up: false,
                down: false,
            },
        }
    }
}
impl CameraController {
    pub fn process_keyboard(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
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
                        self.input_state.forward = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.input_state.left = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.input_state.backward = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.input_state.right = is_pressed;
                        true
                    }
                    VirtualKeyCode::Z => {
                        self.input_state.up = is_pressed;
                        true
                    }
                    VirtualKeyCode::X => {
                        self.input_state.down = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
    // pub fn mouse_look(&mut self, cam: &mut Camera, delta_x: f32, delta_y: f32) {
    //     // let sensitivity = 0.01;
    //     // let right_vec = glm::vec3(
    //     //         cam.view_direction.z,
    //     //         cam.view_direction.y,
    //     //         -cam.view_direction.x,
    //     //     );
    //     // cam.view_direction = glm::rotate_vec3(&cam.view_direction, -delta_x * sensitivity, &cam.up_vector);
    //     // cam.view_direction = glm::rotate_vec3(&cam.view_direction, delta_y * sensitivity, &right_vec);
    // }
    pub fn update(&mut self, cam: &mut Camera) {
        if self.input_state.forward {
            self.move_forward(cam)
        }
        if self.input_state.left {
            self.move_left(cam)
        }
        if self.input_state.backward {
            self.move_backward(cam)
        }
        if self.input_state.right {
            self.move_right(cam)
        }
        if self.input_state.up {
            self.move_up(cam)
        }
        if self.input_state.down {
            self.move_down(cam)
        }
    }
    fn move_forward(&mut self, cam: &mut Camera) {
        cam.eye_posn += self.speed * cam.view_direction;
    }
    fn move_backward(&mut self, cam: &mut Camera) {
        cam.eye_posn -= self.speed * cam.view_direction;
    }
    fn move_left(&mut self, cam: &mut Camera) {
        cam.eye_posn -= self.speed
            * glm::vec3(
                cam.view_direction.y,
                -cam.view_direction.x,
                // cam.view_direction.z,
                0.,
            );
    }
    fn move_right(&mut self, cam: &mut Camera) {
        cam.eye_posn += self.speed
            * glm::vec3(
                cam.view_direction.y,
                -cam.view_direction.x,
                // cam.view_direction.z,
                0.,
            );
    }
    fn move_up(&mut self, cam: &mut Camera) {
        cam.eye_posn += cam.up_vector * self.speed;
    }
    fn move_down(&mut self, cam: &mut Camera) {
        cam.eye_posn -= cam.up_vector * self.speed;
    }
}
