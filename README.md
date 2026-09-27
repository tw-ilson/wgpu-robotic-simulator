# Digital Twin Basics
My project for Northeastern's CS4300 Computer Graphics with Professor Shah.
I built a basic robot simulator which parses a URDF file and can perform forward kinematics to control the joints of the robot.
![2023-12-07-193455_963x730_scrot](https://github.com/tw-ilson/wgpu-robotic-simulator/assets/63574793/aaf6ad26-1de9-4c07-9734-8a86e9e01a1e)

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

To run the XArm example:
> cargo run --example=urdf_arm

## Web deployment

The `urdf_arm` example also builds for the browser (WebGL2 via wgpu's `webgl`
feature). Robot meshes are embedded into the binary, so no asset server is
needed — only the generated `pkg/` directory.

Prerequisites: the wasm target and a `wasm-bindgen-cli` matching the
`wasm-bindgen` crate version in `Cargo.lock` (0.2.118):

> rustup target add wasm32-unknown-unknown
> cargo install wasm-bindgen-cli --version 0.2.118

Build and generate the JS bindings from the repo root:

> cargo build --target wasm32-unknown-unknown --example urdf_arm
> wasm-bindgen target/wasm32-unknown-unknown/debug/examples/urdf_arm.wasm \
>     --out-dir pkg --target web --out-name urdf_arm

Then serve the repo root with any static file server and open `index.html`:

> python3 -m http.server 8080

Notes and current limitations:

- GPU setup is async: on web the example drives it with
  `wasm_bindgen_futures::spawn_local`, on native with `block_on`.
- `wgpu::Features::TEXTURE_BINDING_ARRAY` is only requested on native; the
  WebGL2 fallback device is created without it (no shader in the repo uses it).
- Rayon falls back to sequential execution on `wasm32-unknown-unknown`
  (no threads / `SharedArrayBuffer` required).
- The old toy examples (`stl_parse`, `urdf_dog`, `pixelsort`, `particles`)
  predate the winit 0.29 migration and do not build; only `urdf_arm`,
  `wgpu_triangle`, and `mesh_parse` are kept building.
