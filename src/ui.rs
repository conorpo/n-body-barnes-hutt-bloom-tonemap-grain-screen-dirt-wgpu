use egui_wgpu::*;
use wgpu::{Device, TextureFormat};

use crate::wgpu_state;

struct UI {
    renderer: Renderer,
    screen_descriptor: ScreenDescriptor,
}

impl UI {
    pub fn new(device: &Device, format: TextureFormat, size: [u32; 2]) -> Self {
        let ui_renderer = Renderer::new(&device, format, None, 1);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: 1.0,
        };

        Self { renderer: ui_renderer, screen_descriptor }
    } 

    pub fn render<'rp>(&mut self, rpass: &mut wgpu::RenderPass<'rp>) {

        todo!()
    }
}