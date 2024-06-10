var<private> quad_verts: array<vec2<f32>,6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0)
);

@group(0) @binding(0) var input_texture: texture_2d<f32>;

@vertex fn vs(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
    let pos = vec4<f32>(quad_verts[index], 0.0, 1.0);
    
    return pos;
}

@fragment fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    return textureLoad(input_texture, vec2u(pos.xy), 0);
}