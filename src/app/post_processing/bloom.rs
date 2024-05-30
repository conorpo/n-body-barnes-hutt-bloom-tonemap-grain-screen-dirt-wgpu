use wgpu::*;
use winit::dpi::PhysicalSize;

use crate::config::Config;

pub const MIP_LEVELS: usize = 5;

// Contains the Pipelines and Resources used for Bloom
// Main Bl

pub struct Bloom {
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,
    mipchain: wgpu::Texture
}

impl Bloom {
    pub fn new(device: &Device, config: &Config, format: TextureFormat, size: &PhysicalSize<u32>) -> Bloom {
        
        // Is this where we choose the current mip level?
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Downsample Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let device_size = device.create_buffer(&BufferDescriptor { 
            label: Some("Downsample Device Size Uniform"), 
            size: 8, 
            usage: BufferUsages::UNIFORM, 
            mapped_at_creation: false
        });

        let mipchain =  Bloom::create_bloom_mipchain(device, format, size);

        /*
            Downsample
        */

        // Input Texture, Sampler, Output Size
        let downsample_shader = device.create_shader_module(include_wgsl!("../shaders/downsample.wgsl"));
        let sampling_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bloom Sampling Bindgroup Layout"), //Same bindgroup for downsample and upsample
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0 ,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture { 
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                        multisampled: false
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });

        let downsample_uniform_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bloom Downsample Uniform Bindgroup Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Uniform, 
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
            ]
        });

        let downsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Bloom Downsample Pipeline Descriptor"),
            bind_group_layouts: &[
                &sampling_bindgroup_layout,
                &downsample_uniform_bindgroup_layout,
            ],
            push_constant_ranges: &[]
        });

        let downsample_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Bloom Downsample Pipeline"),
            layout: Some(&downsample_pipeline_layout),
            vertex: VertexState {
                module: &downsample_shader,
                entry_point: "vs",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &downsample_shader,
                entry_point: "fs",
                targets: &[
                    Some(ColorTargetState {
                        format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL
                    })
                ]
            }),
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multiview: None
        });

        /*
            Upsampling
            Takes a texture mip as input, render target is the next biggest mip, until ful res
            Uniforms: Filter Radius
        */

        let upsample_shader = device.create_shader_module(include_wgsl!("../shaders/upsample.wgsl"));

        let upsample_uniform_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Upsample Uniform Bindgroup Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None,
                    },
                    count: None,
                    visibility: ShaderStages::FRAGMENT
                },
            ]
        });

        let upsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Upsample Pipeline Layout"),
            bind_group_layouts: &[
                &sampling_bindgroup_layout,
                &upsample_uniform_bindgroup_layout
            ],
            push_constant_ranges: &[],
        });

        let upsample_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Upsample Pipeline"),
            layout: Some(&upsample_pipeline_layout),
            vertex: VertexState {
                module: &upsample_shader,
                entry_point: "vs",
                buffers: &[]
            },
            fragment: Some(FragmentState {
                module: &upsample_shader,
                entry_point: "fs",
                targets: &[Some(ColorTargetState {
                    write_mask: ColorWrites::ALL,
                    format: format,
                    blend: None // Should we do blending here or in shader?
                })],
            }),
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multiview: None
        });

        Self {
            downsample_pipeline: downsample_pipeline,
            upsample_pipeline: upsample_pipeline,
            mipchain: mipchain
        }
    }

    pub fn create_bloom_mipchain(device: &Device, format: TextureFormat, physical_size: &PhysicalSize<u32>) -> Texture {
        device.create_texture(&TextureDescriptor { 
            label: Some("bloom_mipmap"), 
            size: Extent3d {
                width: physical_size.width,
                height: physical_size.height,
                depth_or_array_layers: 1
            }, 
            mip_level_count: MIP_LEVELS as u32, 
            sample_count: 1, 
            dimension: TextureDimension::D2, 
            format: format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING, 
            view_formats: &[] 
        })
    }

    pub fn render<'rp>(&self, rpass: &mut wgpu::RenderPass<'rp>) {
        
    }
}