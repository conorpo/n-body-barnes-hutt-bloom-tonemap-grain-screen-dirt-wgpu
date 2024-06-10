use wgpu::*;
use winit::dpi::PhysicalSize;

use crate::config::Config;

/*
    Simply renders a texture to the output screen texture,
    will most likely not be needed
*/


pub struct Present {
    bindgroup_layout: BindGroupLayout,
    bindgroup: BindGroup,
    pipeline: RenderPipeline,
}

impl Present {
    pub fn new(device: &Device, _config: &Config, format: TextureFormat,  size: &PhysicalSize<u32>, texture_view: &TextureView) -> Self {
        let bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Presentation Bindgroup Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture { 
                    sample_type: TextureSampleType::Float { filterable: false }, 
                    view_dimension: TextureViewDimension::D2, 
                    multisampled: false 
                },
                count: None
            }]
        });

        let bindgroup = Present::create_bind_group(device, &bindgroup_layout, texture_view);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Presentation Pipeline Layout"),
            bind_group_layouts: &[
                &bindgroup_layout
            ],
            push_constant_ranges: &[]
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/present.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Presentation Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
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

        return Self {
            bindgroup_layout: bindgroup_layout,
            bindgroup: bindgroup,
            pipeline: pipeline
        }
    }

    pub fn create_bind_group(device: &Device, layout: &BindGroupLayout, texture_view: &TextureView) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Present Shader Bindgroup"),
            layout: layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(texture_view),
            }]
        })
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bindgroup, &[]);
        rpass.draw(0..6, 0..1);
    }

    pub fn recreate_bindgroup(&mut self, device: &Device, texture_view: &TextureView) {
        self.bindgroup = Present::create_bind_group(device, &self.bindgroup_layout, texture_view);
    }
}