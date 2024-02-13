use crate::{camer_control, level::{level::{LevelData, LevelState}, mesh::{Mesh, Meshable}, room::Room}, more_stolen_code::FileDialog, stolen_code_to_update_dependencies, renderer::{self, camera::Camera, texture::TextureData}};
use egui::{Context, FontFamily, FontId, RichText, vec2, Button, Color32};
use egui_modal::Modal;
use instant::Instant;
use std::{collections::HashMap, fmt::format, path::PathBuf};
use winit::{event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};
use egui_dnd;
use cgmath::{Point3, Rad};
use egui::FontDefinitions;
use stolen_code_to_update_dependencies::{Platform, PlatformDescriptor};
use renderer::renderstate::State;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::{Fullscreen, Window, WindowBuilder},
};

use super::game_folder_structure::GameData;

pub struct ApplicationState{
    screen_state: ScreenState,
    pub interacting_with_ui: bool,
    cursor_inside:bool,
    default_tex:TextureData,
    render_state:State,
    level_state:LevelState,
    last_render_time:Instant,
    platform:Platform,
}


pub enum ScreenState {
    MainMenu {
        opened_file: Option<PathBuf>,
        open_file_dialog: Option<FileDialog>,
        game_data: Option<GameData>,
        create_new:bool
    },
    Editor{
        editor_state:EditorState,
        game_data:GameData,
        folder_path:PathBuf,
    },
}

#[derive(Clone)]
pub enum EditorState{
    LevelSelection{
        possible_new_level_names:HashMap<String,String>,
        selected_level:Option<String>,
    },
    LevelEditing{
        selected_level:String,
    },
}

impl ApplicationState {
    pub async fn new(event_loop:&EventLoop<()>) -> Self {
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
    
        
        let render_state: State = State::new(window, vec![], vec![]).await;
        let default_tex = TextureData::new(&render_state.default_texture, "default".into());
        let mut level = LevelData::new(&default_tex);
        level.start_camera = camer_control::CameraController::new(
            4.0,
            0.4,
            Camera::new(Point3::new(0.0, 0.0, -10.0), Rad(0.0), Rad(0.0)),
        );
        let level_state = LevelState::from_level_data(&level);
    
        let size = render_state.size;
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: render_state.window().scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
    
        let last_render_time = instant::Instant::now();
        Self {
            screen_state: ScreenState::MainMenu {
                opened_file: None,
                open_file_dialog: None,
                game_data: None,
                create_new:false,
            },
            default_tex:default_tex,
            cursor_inside:false,
            interacting_with_ui: true,
            render_state,
            last_render_time,
            level_state,
            platform,
        }
    }
    pub fn ui(&mut self, ctx: &Context) {
        if let ScreenState::MainMenu { opened_file:Some(folder_path), game_data:Some(game_data),.. } = &self.screen_state{
            self.screen_state = ScreenState::Editor{
                editor_state:EditorState::LevelSelection{
                    selected_level:None,
                    possible_new_level_names:{
                        let mut h = HashMap::new();
                        game_data.levels.iter().for_each(|level_name|{
                            h.insert(level_name.clone(), level_name.clone());
                        });
                        h
                    }
                },
                game_data: game_data.clone(),
                folder_path: folder_path.to_path_buf()
                };
        }
        match &mut self.screen_state {
            ScreenState::MainMenu {
                opened_file,
                open_file_dialog,
                game_data,
                create_new
            } => {
                egui::CentralPanel::default().show(&ctx, |ui| {
                    ui.add_space(30.0);
                    ui.heading(
                        RichText::new("Main Menu")
                            .strong()
                            .font(FontId::monospace(150.0)),
                    );
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::new(24.0, FontFamily::Proportional),
                    );
                    ui.add_space(10.0);
                    if ui.button("Create New").clicked() {
                        *create_new = true;
                        let mut dialog = FileDialog::select_folder(opened_file.clone());
                        dialog.open();
                        *open_file_dialog = Some(dialog);
                    };
                    ui.add_space(10.0);
                    if ui.button("Edit Existing").clicked() {
                        let mut dialog = FileDialog::select_folder(opened_file.clone());
                        dialog.open();
                        *open_file_dialog = Some(dialog);
                    };

                    let mut modal = Modal::new(ctx, "error");

                    if let Some(dialog) = open_file_dialog {
                        if dialog.show(ctx).selected() {
                            if let Some(file) = dialog.path() {
                                if *create_new{
                                    *game_data = Some(GameData::new());
                                    *opened_file = Some(file.to_path_buf());
                                    game_data.as_mut().expect("How. I litteraly just asigned a Some value to this var")
                                        .generate_new_game_folder(file.to_path_buf())
                                        .expect("I alrealdy checked and this worked earlier");
                                }else{
                                    if let Some(game_data_from_path) =
                                        GameData::generate(&file.to_path_buf()) 
                                    {
                                        *opened_file = Some(file.to_path_buf());
                                        *game_data = Some(game_data_from_path);
                                    } else {
                                        modal
                                            .dialog()
                                            .with_title("Invalid Folder")
                                            .with_body(format!("Folder {} structure isn't correct or the config file is messed up :(",file.display()))
                                            .open();
                                    }
                                }
                            }
                        }
                    }
                    modal.show_dialog();
                });
            }
            ScreenState::Editor { editor_state, game_data, folder_path } => {
                
                match editor_state{
                    EditorState::LevelSelection{
                        possible_new_level_names,
                        selected_level
                    }=>{
                        egui::TopBottomPanel::top("top").show(ctx, |ui|{
                            ui.horizontal(|ui|{
                                if ui.button("create level").clicked(){
                                    let num = game_data.levels.iter().fold(0, |acc, level_name|{
                                        if let Some(suffix) = level_name.strip_prefix("new_level_"){
                                            if let Ok(num) = suffix.parse::<u32>(){
                                                if num>acc{
                                                    return num;
                                                }
                                            }
                                        }
                                        return acc;
                                    });
                                    game_data.levels.push(format!("new_level_{}",num+1));
                                    game_data.levels_data.insert(format!("new_level_{}",num+1), LevelData::new(&self.default_tex));
                                    possible_new_level_names.insert(format!("new_level_{}",num+1), format!("new_level_{}",num+1));
                                }
                                if ui.button("save").clicked(){
                                    game_data.update_config();
                                    let _ = game_data.generate_new_game_folder(folder_path.clone());
                                }
                            });
                        });
                        egui::CentralPanel::default().show(ctx, |ui|{
                            ui.set_width(ui.available_width());
                            ui.horizontal_wrapped(|ui|{
                                let mut levels_to_remove = vec![];
                                let cloned_level_names =  game_data.levels.clone();
                                egui_dnd::dnd(ui, "level_moving").show_vec_sized(&mut game_data.levels,vec2(120.0, 90.0), |ui, level, handle, _state|{
                                    handle.ui(ui, |ui|{
                                        ui.set_width(ui.available_width());
                                        ui.set_height(ui.available_height());
                                        ui.add_sized(vec2(120.0, 90.0), Button::new(level.clone())).context_menu(|ui|{
                                            if ui.button("Open").clicked(){
                                                *selected_level = Some(level.clone());
                                            }
                                            ui.horizontal(|ui|{
                                                let response = ui.button("Rename");
                                                ui.text_edit_singleline(possible_new_level_names.get_mut(&level.clone()).unwrap());
                                                if response.clicked(){
                                                    if !cloned_level_names.iter().any(|level_name|{
                                                        *level_name == possible_new_level_names[level]
                                                    }){
                                                        let old_name = level.clone(); 
                                                        *level = possible_new_level_names.get(level).unwrap().clone();
                                                        game_data.levels_data.insert(level.clone(), game_data.levels_data[&old_name].clone());
                                                        game_data.levels_data.remove(&old_name);
                                                        possible_new_level_names.remove(&old_name);
                                                        possible_new_level_names.insert(level.clone(),level.clone());
                                                    }
                                                }
                                            });
                                            if ui.add(Button::new(RichText::new("Delete").color(Color32::BLACK).text_style(egui::TextStyle::Heading).strong()).fill(Color32::LIGHT_RED)).clicked() {
                                                possible_new_level_names.remove(&level.clone());
                                                levels_to_remove.push(level.clone());
                                                ui.close_menu();
                                            }
                                        });
                                    });
                                });
                                levels_to_remove.into_iter().for_each(|level_to_remove|{
                                    game_data.levels.retain(|level_name|{*level_name!=level_to_remove});
                                });
                            });
                        });
                    },  
                    EditorState::LevelEditing{
                        selected_level,
                    }=>{
                        let level = &mut self.level_state;
                        egui::SidePanel::left("selector").show_animated(ctx,self.interacting_with_ui, |ui|{
                            egui::ScrollArea::new([false,true]).show(ui, |ui|{
                                ui.collapsing(RichText::new("Rooms").heading(), |ui|{
                                    level.rooms.iter().for_each(|room|{
                                        ui.collapsing(format!("Room: {}",&room.name), |ui|{
                                            room.moddifiers.iter().for_each(|moddifier|{
                                                ui.label(match &moddifier{
                                                    crate::level::room::Modifier::Ramp { .. } => "Ramp",
                                                    crate::level::room::Modifier::Cliff { .. } => "Extend",
                                                    crate::level::room::Modifier::Disc { .. } => "Platform",
                                                });
                                            })
                                        });
                                    });
                                });
                            });
                        });
                    }
                }
            },
        }
    }
    pub fn input_device(&mut self, event: &DeviceEvent, ){
        match event {
            DeviceEvent::MouseMotion { delta }=>{
                if !self.interacting_with_ui && self.cursor_inside{
                    self.level_state.camera_controler.process_mouse(delta.0, delta.1);
                }
            },
            _ =>{},
        }
    }
    pub fn input_window(&mut self, event: &WindowEvent, is_event_captured:bool) {
        //Load new level into level_sate
        {
            if let ApplicationState{screen_state:ScreenState::Editor { editor_state,game_data,..},..} = self{
                if let EditorState::LevelSelection { selected_level:Some(selected_level),.. } = editor_state.clone() {
                    *editor_state = EditorState::LevelEditing { selected_level: selected_level.clone() };
                    self.level_state = LevelState::from_level_data(&game_data.levels_data[&selected_level]);
                }
            }
        }
        match event {
            WindowEvent::CursorEntered { .. }=>{
                self.cursor_inside = true;
            }
            WindowEvent::CursorLeft { .. }=>{
                self.cursor_inside = false;
            }
            WindowEvent::KeyboardInput{event:KeyEvent{state,physical_key,..},..}=>{
                if let PhysicalKey::Code(key_code) = physical_key{
                    if !self.interacting_with_ui{
                        self.level_state.camera_controler.process_keybord(key_code, state);
                    }
                    match key_code {
                        KeyCode::Escape=>{
                            self.interacting_with_ui = true;
                            self.level_state.camera_controler.remove_velocity();
                        }
                        _=>{}
                    }
                }
            },
            
            WindowEvent::MouseInput { state:ElementState::Pressed, button:MouseButton::Left,.. } if self.interacting_with_ui => {
                if !is_event_captured {
                    self.interacting_with_ui = false;
                }
            }
            _=>{},
        }
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
    pub async fn run(mut self, event_loop:EventLoop<()>) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }
        
        egui_extras::install_image_loaders(&self.platform.context());
        let _ = event_loop.run(move |winit_event, control_flow| {
            let is_event_captured = self.platform.captures_event(&winit_event);
            self.platform.handle_event(&winit_event);
            self.render_state
                .window()
                .set_cursor_visible(self.interacting_with_ui);
            match winit_event {
                Event::DeviceEvent { event, .. } => {
                    self.input_device(&event);
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.render_state.window().id() => {
                    self.input_window(event, is_event_captured);
                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            self.render_state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            self.render_state.resize(self.render_state.window().inner_size());
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
                        self.render_state.window().toggle_fullscreen();
                    }
                    if *event == WindowEvent::RedrawRequested {
                        let now = instant::Instant::now();
                        let dt = now - self.last_render_time;
                        self.last_render_time = now;
    
                        self.platform.begin_frame();
    
                        let full_output = {
                            let ctx = self.platform.context();
                            self.ui(&ctx);
                            ctx.end_frame()
                        };
                        
                        self.render_state.update(dt, &mut self.level_state.camera_controler);
                        self.level_state.update();
                        let meshs: Vec<Mesh> = self.level_state.mesh();
                        if self.render_state.window().is_visible().unwrap_or(true){
                            match self.render_state.render(meshs, full_output, &self.platform) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => self.render_state.resize(self.render_state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                    }
                    }
                }
                _ => {}
            }
            self.render_state.window().request_redraw();
        });
    }
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
