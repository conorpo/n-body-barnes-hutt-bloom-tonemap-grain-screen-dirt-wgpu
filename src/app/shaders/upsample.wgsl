struct UpsampleSettings {
    filter_size: f32,
    mip_level: f32,
}
@group(0) @binding(0) var input_texture: texture_2d;
@group(0) @binding(1) var samp: sampler;

@group(1) @binding(0) var<uniform> upsample_settings: UpsampleSettings;

const quad_verts = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0)
);

@vertex fn vs(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
    let pos = vec4<f32>(quad_verts[index], 0.0, 1.0);
    return pos;
}

@fragment fn fs(@builtin(position) uv: vec4f) -> @location(0) vec4f {
    //Upsample + Filter
    let input_dim = textureDimensions(input_mip);
    let texcoord = uv.xy / downsample_settings.output_size;  

    let d = upsample_settings.filter_size;
    let level = upsample_settings.mip_level;

    //9 Samples, Tent Filter
    //
    //  a b c
    //  d e f
    //  g h i
    //

    let a = textureSampleLevel(input_texture, samp, texcoord + vec2(-d, -d), level);
    let b = textureSampleLevel(input_texture, samp, texcoord + vec2(0, -d), level);
    let c = textureSampleLevel(input_texture, samp, texcoord + vec2(d, -d), level);

    let d = textureSampleLevel(input_texture, samp, texcoord + vec2(-d, 0), level);
    let e = textureSampleLevel(input_texture, samp, texcoord + vec2(0, 0), level);
    let f = textureSampleLevel(input_texture, samp, texcoord + vec2(d, 0), level);

    let g = textureSampleLevel(input_texture, samp, texcoord + vec2(-d, d), level);
    let h = textureSampleLevel(input_texture, samp, texcoord + vec2(0, d), level);
    let i = textureSampleLevel(input_texture, samp, texcoord + vec2(d, d), level);

    let weighted_average = e * 4.0;
    weighted_average += (b+d+f+h) * 2.0;
    weighted_average += (a + c + g + i);
    weighted_average /= 16.0;

    return weighted_average;
}