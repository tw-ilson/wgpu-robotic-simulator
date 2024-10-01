use wgpu_robotic_simulator::graphics::GraphicsProgram;
use wgpu_robotic_simulator::wgpu_program::WGPUGraphics;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let program = WGPUGraphics::new(200, 200, &window);
    program.get_backend_info()
}
