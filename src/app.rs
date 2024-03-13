use wgpu::core::device::queue;
use wgpu::hal::auxil::db;
use wgpu::RenderPipeline;

use crate::config::Config;
use crate::wgpu_state::{self, WgpuState};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use nalgebra::{Matrix4, Vector4};

mod galaxy;
use galaxy::Galaxy;

mod render;
use render::Renderer;

mod camera;
use camera::{Camera, PerspectiveProjection, Projection};

pub struct AppState<'window> {
    pub wgpu_state: WgpuState<'window>,
    pub galaxy: Galaxy,
    pub renderer: Renderer,
    pub camera: Camera<PerspectiveProjection>,
}

impl<'window> AppState<'window> {
    pub fn new(wgpu_state: WgpuState<'window>, config: &Config) -> Self {
        let galaxy = Galaxy::new(&wgpu_state.device, config);

        let camera = Camera::<PerspectiveProjection>::new(&wgpu_state.device);
        
        dbg!(camera.get_view_matrix());
        dbg!(camera.projection.get_projection_matrix());

        for i in 0..10 {
            let star = galaxy.stars[i];
            dbg!(star.position);


            let homogenous = Vector4::new(star.position[0], star.position[1], star.position[2], 1.0);
            let transformed = camera.projection.get_projection_matrix() *  camera.get_view_matrix() * homogenous;

            dbg!(transformed);
        }

        let renderer = Renderer::new(&wgpu_state.device, config, &galaxy, wgpu_state.config.format, &camera);

        Self {
            wgpu_state,
            galaxy,
            renderer,
            camera,
        }
    }   

    pub fn update(&self) {
        self.camera.update(&self.wgpu_state.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.wgpu_state.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.renderer.render(&mut render_pass, &self.galaxy);
        }

        self.wgpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
