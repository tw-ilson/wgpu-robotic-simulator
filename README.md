# Digital Twin Basics
For my project for Northeastern's CS4300 Computer Graphics with Professor Shah, I built a basic robot simulator which parses a URDF file and can perform forward kinematics to control the joints of the robot.

The crate provides several modules:
 - `wgpu_program` provides a simple engine for rendering meshes and scene graphs
 - `urdf` parses URDF XML into a scene graph with transformation information, as well as visual, inertial, collision data
 - `geometry` provides mesh parsing and homogeneous transformations
 - `shader` convenience traits for compiling shader programs
 - `bindings` convenience traits for creating bindings to buffers in the program
 - `camera` data structure for creating camera
 - `light` data structure for adding lights
 - `texture` convenience for creating textures

## build and run
To build the library with an up-to-date Rust toolchain:
> cargo build

To run:
> cargo run --example=urdf_parse
