@group(0) @binding(0) var input_mip: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

var<private> quad_verts: array<vec2<f32>,6> =  array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0)
);

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

// Note, you can use the vertex shader to get texcoords (0,1), 
//  no need to pass the render target size



@fragment fn fs(input: VertexOutput) -> @location(0) vec4f {   
    //Use TextureDimension with miplevel instead..
    let input_dim = textureDimensions(input_mip);
    let pixel_size = 1.0 / vec2f(input_dim);

    let texcoord = input.texcoord;

    let a = textureSample(input_mip, samp, texcoord + vec2(-2*pixel_size.x, -2*pixel_size.y));
    let b = textureSample(input_mip, samp, texcoord + vec2(0, -2*pixel_size.y));
    let c = textureSample(input_mip, samp, texcoord + vec2(2*pixel_size.x, -2*pixel_size.y));

    let d = textureSample(input_mip, samp, texcoord + vec2(-1*pixel_size.x, -1*pixel_size.y));
    let e = textureSample(input_mip, samp, texcoord + vec2( 1*pixel_size.x, -1*pixel_size.y));
    
    let f = textureSample(input_mip, samp, texcoord + vec2(-2*pixel_size.x, 0));
    let g = textureSample(input_mip, samp, texcoord + vec2(0,0));
    let h = textureSample(input_mip, samp, texcoord + vec2( 2*pixel_size.x, 0));

    let i = textureSample(input_mip, samp, texcoord + vec2(-1*pixel_size.x, 1*pixel_size.y));
    let j = textureSample(input_mip, samp, texcoord + vec2( 1*pixel_size.x, 1*pixel_size.y));

    let k = textureSample(input_mip, samp, texcoord + vec2(-2*pixel_size.x, 2*pixel_size.y));
    let l = textureSample(input_mip, samp, texcoord + vec2(0, 2*pixel_size.x));
    let m = textureSample(input_mip, samp, texcoord + vec2( 2*pixel_size.x, 2*pixel_size.y));
    
    let weighted_average = (d + e + i + j) * 0.125 + //red
                           (g) * 0.125 + // center (overlap of 4)
                           (a + c + k + m) * 0.03125 +// corners (no overlap)
                           (b + f + h + l) * 0.0625; // edges (overlap of 2)

    return weighted_average;
}