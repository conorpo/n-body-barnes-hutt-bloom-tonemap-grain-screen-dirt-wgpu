use wgpu::*;
use winit::dpi::PhysicalSize;

use crate::{config::Config};


// Contains the Pipelines and Resources used for Bloom

/*
TODO
    Blend the bloom shader
    Find good filter size
    Add UI controls
*/

pub const MIP_LEVELS: usize = 5;
pub struct Bloom {
    pub mipchain: wgpu::Texture,
    pub mipchain_views: Vec<TextureView>,

    sampler: Sampler,
    sampling_bindgroup_layout: BindGroupLayout,
    sampling_bindgroups: Vec<BindGroup>,

    downsample_pipeline: wgpu::RenderPipeline,

    upsample_pipeline: wgpu::RenderPipeline,
    upsample_settings: wgpu::Buffer,
    upsample_settings_bindgroup: wgpu::BindGroup,
}

impl Bloom {
    pub fn new(device: &Device, _config: &Config, format: TextureFormat, size: &PhysicalSize<u32>) -> Bloom {
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Downsample Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let (mipchain, mipchain_views) =  Bloom::create_mipchain_and_views(device, format, size);

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

        let sampling_bindgroups = Self::create_sampling_bindgroups(device, &sampler, &sampling_bindgroup_layout, &mipchain_views);
        
        
        /*
            Downsample
        */
        let downsample_shader = device.create_shader_module(include_wgsl!("../shaders/downsample.wgsl"));
        
        let downsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Bloom Downsample Pipeline Descriptor"),
            bind_group_layouts: &[
                &sampling_bindgroup_layout,
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

        let upsample_settings = device.create_buffer(&BufferDescriptor { 
            label: Some("Upsample Settings Uniform"),
            size: 4,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let upsample_settings_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Upsample Settings Bindgroup Layout"),
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

        let upsample_settings_bindgroup = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bloom Upsample Settings Bindgroup"),
            layout: &upsample_settings_bindgroup_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &upsample_settings,
                        offset: 0,
                        size: None
                    })
                }
            ],
        });

        let upsample_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Upsample Pipeline Layout"),
            bind_group_layouts: &[
                &sampling_bindgroup_layout,
                &upsample_settings_bindgroup_layout
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
                    blend: Some(BlendState {
                        color: BlendComponent {
                            operation: BlendOperation::Add,
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One
                        },
                        alpha: BlendComponent {
                            operation: BlendOperation::Add,
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One
                        }
                    })
                })],
            }),
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multiview: None
        });

        Self {
            mipchain: mipchain,
            mipchain_views: mipchain_views,
            
            sampler: sampler,
            sampling_bindgroup_layout: sampling_bindgroup_layout,
            sampling_bindgroups: sampling_bindgroups,
            
            downsample_pipeline: downsample_pipeline,

            upsample_pipeline: upsample_pipeline,
            upsample_settings_bindgroup: upsample_settings_bindgroup,
            upsample_settings: upsample_settings
        }
    }

    // The create_resources functions are always static, enabling use for initialization aswell as recreation
    pub fn create_mipchain_and_views(device: &Device, format: TextureFormat, physical_size: &PhysicalSize<u32>) -> (Texture, Vec<TextureView>) {
        let new_mipchain = device.create_texture(&TextureDescriptor { 
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
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST, 
            view_formats: &[] 
        });

        let new_mipchain_views = (0..MIP_LEVELS).map(|i: usize| {
            new_mipchain.create_view(&TextureViewDescriptor {
                label: Some("Bloom Mipchain Main View"),
                format: Some(format),
                dimension: Some(TextureViewDimension::D2),
                aspect: TextureAspect::All,
                base_mip_level: i as u32,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
            })
        }).collect();

        (new_mipchain, new_mipchain_views)
    }

    pub fn create_sampling_bindgroups(device: &Device, sampler: &Sampler, layout: &BindGroupLayout, views: &Vec<TextureView>) -> Vec<BindGroup> {
        (0..MIP_LEVELS).map(|i| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Bloom Sampling Bindgroup"),
                layout: layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&views[i])
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    }
                ]
            })
        }).collect()
    }

    pub fn recreate_mipchain_and_bindgroups(&mut self, device: &Device, format: TextureFormat, physical_size: &PhysicalSize<u32>) -> () {
        self.mipchain.destroy(); //Also "destroys" the texture views and bindgroups, in that everything will be cleaned up when those references are dropped

        (self.mipchain, self.mipchain_views) = Self::create_mipchain_and_views(device, format, physical_size);

        self.sampling_bindgroups = Self::create_sampling_bindgroups(device, &self.sampler, &self.sampling_bindgroup_layout, &self.mipchain_views);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue) {
        //Downsampling
        for i in 0..(MIP_LEVELS-1) {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Bloom Downsample Renderpass {} -> {}", i, i + 1)),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.mipchain_views[i+1],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            });

            render_pass.set_pipeline(&self.downsample_pipeline);
            render_pass.set_bind_group(0, &self.sampling_bindgroups[i], &[]);
            render_pass.draw(0..6, 0..1);   
            
            /*
                Either find a way to do downsampling / upsampling with a new render pass for each mip-level, because otherwise all the bindgroup optimization doesnt matter. Possible solution is to use compute shaders, but in that case output goes through UAV, which might be slightly slower than a fragment shader, not sure. Also would have to bind output textures. If we stick with the multiple render pass solution, then no point in binding the whole mipchain each time, only bind needed input level.
            */
        }

        //Upsampling
        let filter_size = 1.0f32;
        let buffer_data_bytes = [filter_size.to_be_bytes()].concat();
        queue.write_buffer(&self.upsample_settings, 0, &buffer_data_bytes);
        for i in (1..MIP_LEVELS).rev() {
            // Update Settings Buffer, Update Render Target, Create Render Pass, Draw
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Bloom Upsample Renderpass {} -> {}", i, i - 1)),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.mipchain_views[i-1],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            });

            render_pass.set_pipeline(&self.upsample_pipeline);
            render_pass.set_bind_group(0, &self.sampling_bindgroups[i], &[]);
            render_pass.set_bind_group(1, &self.upsample_settings_bindgroup, &[]);
            render_pass.draw(0..6, 0..1);   
        }
        //At this point bloom output is in miplevel 0 
    }
}