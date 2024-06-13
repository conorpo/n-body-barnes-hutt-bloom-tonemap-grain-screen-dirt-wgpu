use std::time;

use egui_wgpu::ScreenDescriptor;
use env_logger::fmt::Timestamp;
use hal::auxil::db;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::KeyEvent;
use bytemuck;
use nalgebra::{Matrix4, Vector4};

use crate::config::Config;
use crate::wgpu_state::{self, WgpuState};
use crate::ui::UI;

pub mod galaxy;
use galaxy::Galaxy;

mod render;
use render::Renderer;

pub mod camera;
use camera::{Camera, PerspectiveProjection, Projection};

pub mod post_processing;
use post_processing::bloom::Bloom;
use post_processing::present::Present;

pub mod timestamps;
use timestamps::Timestamps;


pub struct AppState<'window> {
    config: Config,

    pub wgpu_state: WgpuState<'window>,
    pub galaxy: Galaxy,
    pub renderer: Renderer,
    pub camera: Camera<PerspectiveProjection>,

    // Post processing
    pub bloom: Bloom,
    pub present: Present,

    //UI
    pub ui: UI,

    //Profiling
    timestamps: Timestamps
}

impl<'window> AppState<'window> {
    
    pub fn new(wgpu_state: WgpuState<'window>, config: Config, size: &PhysicalSize<u32>) -> Self {
        // Simulation
        let galaxy = Galaxy::new(&wgpu_state.device, &config);

        // Primary Rendering
        let camera = Camera::<PerspectiveProjection>::new(&wgpu_state.device, &config);
        let renderer = Renderer::new(&wgpu_state.device, &config, wgpu_state.config.format, &camera);

        // Post-Porcessing
        let bloom = Bloom::new(&wgpu_state.device, &config, wgpu_state.config.format, size);
        let present = Present::new(&wgpu_state.device, &config, wgpu_state.config.format, size, &bloom.mipchain_views[0]);

        let ui = UI::new(&wgpu_state.device, wgpu_state.config.format, &wgpu_state.window);
        let timestamps = Timestamps::new(&wgpu_state.device);

        Self {
            wgpu_state,
            galaxy,
            renderer,
            camera,
            bloom,
            present,
            ui,
            timestamps,
            config
        }
    }   

    pub fn update(&mut self) {
        self.galaxy.add_stars(&self.config, &self.wgpu_state.queue, 200);
        self.camera.update(&self.wgpu_state.queue);
    }

    //Move this to Renderer
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.wgpu_state.surface.get_current_texture()?;

        
        let mut encoder = self.wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom.mipchain_views[0],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    }
                })], 
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: Some(RenderPassTimestampWrites {
                    query_set: &self.timestamps.query_set,
                    beginning_of_pass_write_index: Some(0),
                    end_of_pass_write_index: Some(1)
                }),
            });

            
            self.renderer.render(&mut render_pass, &self.galaxy);
        }

        // Bloom
        self.bloom.render(&mut encoder, &self.wgpu_state.queue);

        //Present
        let output_view = output.texture.create_view(&TextureViewDescriptor::default());//&self.bloom.mipchain_views[0];
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    }
                })], 
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: Some(RenderPassTimestampWrites {
                    query_set: &self.timestamps.query_set,
                    beginning_of_pass_write_index: Some(2),
                    end_of_pass_write_index: Some(3)
                }),
            });

            self.present.render(&mut render_pass);
        }

        //UI (maybe figure out some abtraction instead of passing all used structs, maybe just pass the specific parameters)
        let ui_output = self.ui.update(&self.wgpu_state, &mut self.camera, &mut self.bloom, &self.timestamps, &self.galaxy);

        let size: [u32;2] = self.wgpu_state.window.inner_size().into();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: 1.0,
        };
        self.ui.render(&mut encoder, &self.wgpu_state, screen_descriptor, &output_view, ui_output);

        
        //Profiling
        self.timestamps.resolve(&mut encoder, &self.wgpu_state.device);

        self.wgpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.timestamps.update_times(&mut self.wgpu_state.device);

        Ok(())
    }

    pub fn keyboard_input(&mut self, event: &KeyEvent) {
        use winit::keyboard::{PhysicalKey, KeyCode};
        if event.state == winit::event::ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.camera.orbit_horizontal(-self.camera.orbit_speed);
                },
                PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.camera.orbit_horizontal(self.camera.orbit_speed);
                },
                PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.camera.orbit_vertical(self.camera.orbit_speed);
                },
                PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.camera.orbit_vertical(-self.camera.orbit_speed);
                },
                _ => {}
            }
        }
    }
    
    pub fn mouse_wheel_input(&mut self, delta: &winit::event::MouseScrollDelta, phase: &winit::event::TouchPhase) {
        use winit::event::MouseScrollDelta;
        use winit::event::TouchPhase;
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                match phase {
                    TouchPhase::Moved => {
                        self.camera.zoom(-(*y * self.camera.zoom_speed));
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.wgpu_state.resize(new_size);
        self.bloom.recreate_mipchain_and_bindgroups(&self.wgpu_state.device, self.wgpu_state.config.format, new_size);
        //Recreate bindgroup which depends on the mipchain
        self.present.recreate_bindgroup(&self.wgpu_state.device, &self.bloom.mipchain_views[0])
    }
}
