// Vertex shader

//-----CAMERA------
struct CameraUniform {
    view_proj: mat4x4<f32>,
}


//-----Light------
struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

struct Transform {
    tmatrix: mat4x4<f32>,
}


@group(0) @binding(0) 
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> light: Light;

@group(2) @binding(0)
var<uniform> transform: Transform;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.world_normal = model.normal;
    
    out.world_position = (transform.tmatrix * vec4<f32>(model.position, 1.0)).xyz;
    out.clip_position = camera.view_proj * vec4<f32>(out.world_position, 1.0);
    return out;
}

/////////////////////////////////////////////////////////////////////////
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(0.0, dot(in.world_normal, light_dir));
    let diffuse_color = light.color * diffuse_strength;
    /* let half_dir = normalize(view_dir + light_dir); */

    /* let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0); */
    /* let specular_color = specular_strength * light.color; */

    let result = (ambient_color + diffuse_strength ) * vec3(1.0,1.0,1.0);
    return vec4<f32>(result, 1.0);
}
