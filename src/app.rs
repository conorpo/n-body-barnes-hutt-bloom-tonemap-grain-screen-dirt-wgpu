use wgpu::core::device::queue;
use wgpu::RenderPipeline;

use crate::config::Config;
use crate::wgpu_state::{self, WgpuState};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

mod galaxy;
use galaxy::Galaxy;

mod render;
use render::Renderer;

pub struct AppState<'window> {
    pub wgpu_state: WgpuState<'window>,
    pub galaxy: Galaxy,
    pub renderer: Renderer,
}

impl<'window> AppState<'window> {
    pub fn new(wgpu_state: WgpuState<'window>, config: &Config) -> Self {
        let galaxy = Galaxy::new();

        let renderer = Renderer::new(&wgpu_state.device, config, &galaxy, wgpu_state.config.format);

        Self {
            wgpu_state,
            galaxy,
            renderer,
        }
    }   

    pub fn update(&self) {
        todo!()
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

            self.renderer.render(&mut render_pass);
        }

        self.wgpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}