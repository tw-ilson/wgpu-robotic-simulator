use nalgebra_glm as glm;
use physics_engine::graphics::GraphicsProgram;
use physics_engine::wgpu_program::WGPUGraphics;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let program = WGPUGraphics::new(200, 200, &event_loop);
    program.get_backend_info()
}
