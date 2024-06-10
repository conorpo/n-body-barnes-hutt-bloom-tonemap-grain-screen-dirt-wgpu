struct UpsampleSettings {
    filter_size: f32,
}

var<private> quad_verts: array<vec2<f32>,6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0)
);

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

@group(1) @binding(0) var<uniform> upsample_settings: UpsampleSettings;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}

@vertex fn vs(@builtin(vertex_index) index : u32) -> VertexOutput{
    let pos = vec4<f32>(quad_verts[index], 0.0, 1.0);
    
    var vertexOutput: VertexOutput;
    vertexOutput.position = pos;
    vertexOutput.texcoord = (pos.xy / 2.0) + 0.5;
    return vertexOutput;
}

@fragment fn fs(input: VertexOutput) -> @location(0) vec4f {
    //Upsample + Filter
    //Use TextureDimension with miplevel instead..
    let texcoord = input.texcoord;

    let filter_size = upsample_settings.filter_size;

    //9 Samples, Tent Filter
    //
    //  a b c
    //  d e f
    //  g h i
    //

    let a = textureSample(input_texture, samp, texcoord + vec2(-filter_size, -filter_size));
    let b = textureSample(input_texture, samp, texcoord + vec2(0, -filter_size));
    let c = textureSample(input_texture, samp, texcoord + vec2(filter_size, -filter_size));

    let d = textureSample(input_texture, samp, texcoord + vec2(-filter_size, 0));
    let e = textureSample(input_texture, samp, texcoord + vec2(0, 0));
    let f = textureSample(input_texture, samp, texcoord + vec2(filter_size, 0));

    let g = textureSample(input_texture, samp, texcoord + vec2(-filter_size, filter_size));
    let h = textureSample(input_texture, samp, texcoord + vec2(0, filter_size));
    let i = textureSample(input_texture, samp, texcoord + vec2(filter_size, filter_size));

    var weighted_average = e * 4.0;
    weighted_average += (b+d+f+h) * 2.0;
    weighted_average += (a + c + g + i);
    weighted_average /= 16.0;

    return weighted_average;
}