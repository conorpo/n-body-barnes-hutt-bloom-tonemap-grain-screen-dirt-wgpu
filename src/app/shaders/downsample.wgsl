@group(0) @binding(0) var input_mip: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

struct DownsampleSettings {
    output_size: vec2<u32>,
    mip_level: f32,
}

@group(1) @binding(0) var<uniform> downsample_settings: DownsampleSettings;


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
    let input_dim = textureDimensions(input_mip);
    let pixel_size = 1 / input_dim;

    let texcoord = uv.xy / downsample_settings.output_size;  
    
    // 13 billinear samples (36-texels)
    //  a b c
    //   d e
    //  f g h
    //   i j
    //  k l m

    let a = textureSample(input_mip, sampl, texcoord + vec2(-2*pixel_size, -2*pixel_size));
    let b = textureSample(input_mip, sampl, texcoord + vec2(0, -2*pixel_size));
    let c = textureSample(input_mip, sampl, texcoord + vec2(2*pixel_size, -2*pixel_size));

    let d = textureSample(input_mip, sampl, texcoord + vec2(-1*pixel_size, -1*pixel_size));
    let e = textureSample(input_mip, sampl, texcoord + vec2( 1*pixel_size, -1*pixel_size));
    
    let f = textureSample(input_mip, sampl, texcoord + vec2(-2*pixel_size, 0));
    let g = textureSample(input_mip, sampl, texcoord + vec2(0,0));
    let h = textureSample(input_mip, sampl, texcoord + vec2( 2*pixel_size, 0));

    let i = textureSample(input_mip, sampl, texcoord + vec2(-1*pixel_size, 1*pixel_size));
    let j = textureSample(input_mip, sampl, texcoord + vec2( 1*pixel_size, 1*pixel_size));

    let k = textureSample(input_mip, sampl, texcoord + vec2(-2*pixel_size, 2*pixel_size));
    let l = textureSample(input_mip, sampl, texcoord + vec2(0, 2*pixel_size));
    let m = textureSample(input_mip, sampl, texcoord + vec2( 2*pixel_size, 2*pixel_size));
    
    let weighted_average = (d + e + i + j) * 0.125 + //red
                           (g) * 0.125 + // center (overlap of 4)
                           (a + c + k + m) * 0.03125 +// corners (no overlap)
                           (b + f + h + l) * 0.0625 // edges (overlap of 2)

    return weighted_average;
}