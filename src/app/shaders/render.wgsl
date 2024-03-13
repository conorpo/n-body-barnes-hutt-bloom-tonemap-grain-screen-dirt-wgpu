struct StarInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct CameraUniform {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@vertex fn vs_main(in: StarInput) -> VertexOutput {
    var out: VertexOutput;
    let transformed_pos = camera.projection_matrix * camera.view_matrix * vec4<f32>(in.position, 1.0);
    out.clip_position = transformed_pos;
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32>{
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}