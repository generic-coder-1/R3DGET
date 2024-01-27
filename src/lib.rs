pub mod application_state;
pub mod camer_control;
pub mod level;
pub mod more_stolen_code;
pub mod plagerized_code_to_update_dependencies;
pub mod renderer;

use cgmath::{Point3, Rad};
use egui::FontDefinitions;
use plagerized_code_to_update_dependencies::{Platform, PlatformDescriptor};
use renderer::renderstate::State;
use std::vec;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowBuilder},
};

use crate::{
    application_state::application_state::ApplicationState,
    level::{
        level::{LevelData, LevelState},
        mesh::{Mesh, Meshable},
    },
    renderer::camera::Camera,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().expect("couldn't initalize a window");
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("REDG3T");
    window.set_theme(Some(winit::window::Theme::Dark));

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(1280, 720));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut application_state = ApplicationState::new();

    let mut render_state: State = State::new(window, vec![], vec![]).await;
    let mut level = LevelData::new(&("default".into()));
    level.start_camera = camer_control::CameraController::new(
        4.0,
        0.4,
        Camera::new(Point3::new(0.0, 0.0, -10.0), Rad(0.0), Rad(0.0)),
    );
    let mut level_state = LevelState::from_level_data(&level);

    let size = render_state.size;
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width,
        physical_height: size.height,
        scale_factor: render_state.window().scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    let mut last_render_time = instant::Instant::now();

    let _ = event_loop.run(move |winit_event, control_flow| {
        let is_event_captured = platform.captures_event(&winit_event);
        platform.handle_event(&winit_event);
        render_state
            .window()
            .set_cursor_visible(application_state.interacting_with_ui);
        match winit_event {
            Event::DeviceEvent { event, .. } => {
                application_state.input_device(&event, &mut level_state);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == render_state.window().id() => {
                application_state.input_window(event, &mut level_state, is_event_captured);
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        render_state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        render_state.resize(render_state.window().inner_size());
                    }
                    _ => {}
                }
                if let WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::F11),
                            ..
                        },
                    ..
                } = event
                {
                    render_state.window().toggle_fullscreen();
                }
                if *event == WindowEvent::RedrawRequested {
                    let now = instant::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    platform.begin_frame();

                    let full_output = {
                        let ctx = platform.context();
                        application_state.ui(&ctx);
                        ctx.end_frame()
                    };

                    render_state.update(dt, &mut level_state.camera_controler);
                    let meshs: Vec<Mesh> = level_state.mesh();
                    match render_state.render(meshs, full_output, &platform) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            _ => {}
        }
        render_state.window().request_redraw();
    });
}

trait WindowFullScreen {
    fn toggle_fullscreen(&self);
}

impl WindowFullScreen for Window {
    fn toggle_fullscreen(&self) {
        if self.fullscreen().is_some() {
            self.set_fullscreen(None);
        } else {
            self.current_monitor().map(|monitor| {
                monitor.video_modes().next().map(|video_mode| {
                    if cfg!(any(target_os = "macos", unix)) {
                        self.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
                    } else {
                        self.set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
                    }
                })
            });
        }
    }
}
