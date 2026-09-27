use wgpu_robotic_simulator::graphics::GraphicsProgram;
use wgpu_robotic_simulator::wgpu_program::WGPUGraphics;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let program = futures::executor::block_on(WGPUGraphics::new(200, 200, &window));
    match program {
        Ok(program) => program.get_backend_info(),
        Err(e) => eprintln!("failed to initialize graphics: {e:?}"),
    }
}
