mod config;
mod wgpu_state;
mod app;
mod ui;

use wgpu::hal::auxil::db;
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{self, KeyCode, PhysicalKey}, window::{Theme, Window, WindowBuilder}
};

use config::Config;
use wgpu_state::WgpuState;

fn main() {
    // Logging configuration
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("error initializing log");
        } else {
            env_logger::init();
        }
    } 
    
    match pollster::block_on(run()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("main error match: {}", e);
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
async fn run() -> Result<(), winit::error::EventLoopError> {
    let config = Config::get();

    let size = winit::dpi::PhysicalSize::new(
        config.window_config.size[0],
        config.window_config.size[1],
    );

    let event_loop = EventLoop::new().ok().unwrap();
    //let icon = Icon:
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_theme(Some(Theme::Dark))
        .with_title(config.window_config.title.clone())
        .build(&event_loop)
        .unwrap();

    // Setups Adapter, Device, Surface
    let wgpu_state = WgpuState::new(&window, &config).await;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc | {
                let dst = doc.body();
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("couldn't attach canvas to document");
    }

    // Setups the entire rest of the application
    let mut app = app::AppState::new(wgpu_state, config, &size);

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { ref event, window_id } if window_id == app.wgpu_state.window.id() => {
            app.wgpu_state.window.request_redraw();
            match event {
                WindowEvent::CloseRequested | 
                WindowEvent::KeyboardInput { 
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => app.keyboard_input(event),
                
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size);
                },
                WindowEvent::MouseWheel { delta, phase, .. } => {
                    app.mouse_wheel_input(delta, phase);
                },
                WindowEvent::RedrawRequested => {
                    app.update();
            
                    match app.render() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("render error: {}", e);
                        }
                    }
                    },
                    _ => {}
            }  
            app.ui.handle_input(app.wgpu_state.window, event);  
        },
        Event::AboutToWait => {
            // Redraw


            //app.wgpu_state.window.request_redraw();
        },
        _ => {}
    })
}

