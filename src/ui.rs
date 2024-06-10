use egui_wgpu::*;
use wgpu::{CommandEncoder, Device, TextureFormat};

use egui::epaint::Shadow;
use egui::{Context, Visuals};

use egui_winit::State;

use crate::wgpu_state::{self, WgpuState};

pub struct UI {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl UI {
    pub fn new(device: &Device, format: TextureFormat, window: &winit::window::Window) -> Self {
        let context = Context::default();
        let id = context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        context.set_visuals(visuals);

        let state = egui_winit::State::new(context.clone(), id, window, None, None);

        let renderer = Renderer::new(&device, format, None, 1);

        Self { 
            context,
            renderer, 
            state,
        }
    } 

    // pub fn render<'rp>(&'rp self, encoder: CommandEncoder, wgpu_state: &WgpuState, screen_descriptor: ScreenDescriptor) {
    //     let raw_input = self.state.take_egui_input(&window);
    //     let full_output = self.context.run(raw_input, |ui| {
    //         //run_ui(&self.context);
    //     });

    //     self.state.handle_platform_output(wgpu_state.window, full_output.platform_output);

    //     let tris = self
    //     .context
    //     .tessellate(full_output.shapes, full_output.pixels_per_point);

    //     for (id, image_delta) in &full_output.textures_delta.set {
    //         self.renderer
    //             .update_texture(&wgpu_state.device, &wgpu_state.queue, *id, image_delta);
    //     }

    //     self.renderer.render(rpass, &[], &self.screen_descriptor);
    // }
}