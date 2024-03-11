mod config;
mod wgpu_state;
mod app;

use wgpu::hal::auxil::db;
use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{PhysicalKey, KeyCode}, window::{WindowBuilder, Window}
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
    let window = WindowBuilder::new()
        .with_inner_size(size)
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
    let mut app_state = app::AppState::new(wgpu_state, &config);

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { ref event, window_id } if window_id == app_state.wgpu_state.window.id() => match event {
            WindowEvent::CloseRequested | 
            WindowEvent::KeyboardInput { 
                event: KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
                ..
            } => elwt.exit(),
            WindowEvent::Resized(physical_size) => {
                app_state.wgpu_state.resize(*physical_size);
            },
            WindowEvent::MouseInput {..} => {
                app_state.wgpu_state.input(event);
            }
            _ => {}
        },
        Event::AboutToWait => {
            // Redraw

            //app_state.update();
            match app_state.render() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("render error: {}", e);
                }
            }

            app_state.wgpu_state.window.request_redraw();
        },
        _ => {}
    })
}

