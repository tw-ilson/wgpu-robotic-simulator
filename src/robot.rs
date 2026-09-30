use crate::{urdf::RobotDescriptor, wgpu_program::{MeshBuffer, WGPUGraphics}};


pub trait RobotGraphics {
    fn robot_create_mesh_buffers(&mut self, robot: &RobotDescriptor) -> Vec<MeshBuffer>;
    fn robot_assign_mesh_buffers(&mut self, robot: &RobotDescriptor, buffers: &Vec<MeshBuffer>);
    fn draw_robot(&mut self, robot: &RobotDescriptor, buffers: &Vec<MeshBuffer>, pipeline: &wgpu::RenderPipeline);
    fn robot_create_transform_buffers(&mut self, robot: &RobotDescriptor) -> Vec<wgpu::Buffer>;
    fn robot_assign_transform_buffers(
        &mut self,
        robot: &RobotDescriptor,
        buffer: &Vec<wgpu::Buffer>,
    );
    // fn robot_create_bindings(&mut self, /* robot: &RobotDescriptor, */ light_buffer: &wgpu::Buffer, camera_buffer: &wgpu::Buffer, transform_buffer: &Vec<wgpu::Buffer>);
}

impl RobotGraphics for WGPUGraphics<'_> {
    fn robot_create_mesh_buffers(&mut self, robot: &RobotDescriptor) -> Vec<MeshBuffer> {
        self.create_mesh_buffers(robot.visuals().map(|v| &v.geometry))
    }
    fn robot_assign_mesh_buffers(&mut self, robot: &RobotDescriptor, buffers: &Vec<MeshBuffer>) {
        // Warning: order matters! Mesh i is drawn with transform i, and both
        // follow RobotDescriptor::visuals() order.
        std::iter::zip(buffers, robot.visuals()).map(|(buf, v)| self.assign_mesh_buffer(&v.geometry, buf)).collect()
    }
    fn draw_robot(&mut self, robot: &RobotDescriptor, buffers: &Vec<MeshBuffer>, pipeline: &wgpu::RenderPipeline) {
        self.draw_mesh_list(pipeline, &buffers);
    }
    fn robot_create_transform_buffers(&mut self, robot: &RobotDescriptor) -> Vec<wgpu::Buffer> {
        self.create_transform_buffers(robot.visuals().map(|v| v.transform))
    }
    fn robot_assign_transform_buffers(
        &mut self,
        robot: &RobotDescriptor,
        buffers: &Vec<wgpu::Buffer>,
    ) {
        // std::iter::zip(buffers, &robot.links).for_each(|(b,l)| self.assign_uniform(b, &[l.inertial.transform]))
        self.update_transforms(buffers, robot.visuals().map(|v| v.transform))
    }
}
