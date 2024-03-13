use super::camera::{Camera, Projection};
use super::galaxy::{Galaxy, Star};

use crate::config::Config;
use crate::wgpu_state::WgpuState;
use wgpu::*;

pub struct Renderer {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bindgroup: wgpu::BindGroup,
}

impl Renderer {
    pub fn new<T: Projection + Default>(device: &Device, config: &Config, galaxy: &Galaxy, format: TextureFormat, camera: &Camera<T>) -> Self {
        let shader = device.create_shader_module(include_wgsl!("./shaders/render.wgsl"));

        let bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                camera.get_bindgroup_layout_entry(),
            ],
        });

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &bindgroup_layout,
            entries: &[
                camera.get_bindgroup_entry(),
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bindgroup_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Star::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            multiview: None,
        });


        Self {
            render_pipeline,
            bindgroup,
        }
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, galaxy: &'rpass Galaxy) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bindgroup, &[]);
        rpass.set_vertex_buffer(0, galaxy.stars_buffer.slice(..));
        rpass.draw(0..galaxy.stars.len() as u32, 0..1);
    }
}

