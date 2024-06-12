use egui_wgpu::*;
use wgpu::{CommandEncoder, Device, RenderPassColorAttachment, TextureFormat, TextureView};

use egui::epaint::Shadow;
use egui::{Align2, Context, FullOutput, Visuals};

use egui_winit::{update_viewport_info, State};
use winit::event::WindowEvent;

use crate::app::AppState;
use crate::wgpu_state::{self, WgpuState};

use crate::app::camera::*;
use crate::app::post_processing::bloom::*;
use crate::app::timestamps::Timestamps;

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

    pub fn update(&mut self, wgpu_state: &WgpuState, camera: &mut Camera<PerspectiveProjection>, bloom: &mut Bloom, timestamps: &Vec<u64>) -> FullOutput {
        let raw_input = self.state.take_egui_input(wgpu_state.window);
        
        self.context.run(raw_input, |ui| {
            egui::SidePanel::right("Controls")
            .default_width(1000.0)
            .resizable(true)
            .show(&ui, |mut ui| {
                ui.heading("n-body-barnes-hutt");
                ui.group(|ui| {
                    ui.label("Camera");
                    ui.add(egui::Slider::new(&mut camera.spherical_position.r, 5.0..=5000.0).text("Zoom Level"));
                });

                ui.group(|ui| {
                    ui.label("Bloom");
                    ui.add(egui::Slider::new(&mut bloom.filter_size,0.0..=0.003).text("Filter Size"));
                });

                //ui.add(egui::)
        
                ui.end_row();
            });

            //Temp
            egui::Window::new("Times")
            .default_height(1000.0)
            .default_width(600.0)
            .show(&ui, |ui| {
                ui.heading("Times");
                ui.label(format!("Render Time: {}", (timestamps[1] - timestamps[0]) as f64 / 1000.0));
            });
        })
    }

    pub fn handle_input(&mut self, window: &winit::window::Window, event:&WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn render(&mut self, encoder:&mut CommandEncoder, wgpu_state: &WgpuState, screen_descriptor: ScreenDescriptor, output_view: &TextureView, full_output: FullOutput) {
        //update_viewport_info(&mut raw_input.viewports.get_mut(&self.context.viewport_id()).unwrap(), &self.context, &wgpu_state.window);
        self.state.handle_platform_output(wgpu_state.window, full_output.platform_output);

        let tris = self
        .context
        .tessellate(full_output.shapes, full_output.pixels_per_point);
    
        //Update Textures
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&wgpu_state.device, &wgpu_state.queue, *id, image_delta);
        }

        //Update Buffers
        self.renderer.update_buffers(&wgpu_state.device, &wgpu_state.queue, encoder, &tris, &screen_descriptor);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Egui Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None
        });

        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x);
        }
    }
}