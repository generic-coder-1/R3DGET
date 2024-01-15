use crate::{more_stolen_code::FileDialog, level::level::{LevelState, LevelData}};
use egui::{Context, FontFamily, FontId, RichText, vec2, Button, Color32};
use egui_modal::Modal;
use std::{path::PathBuf, collections::HashMap};
use winit::{event::{WindowEvent, ElementState, MouseButton, DeviceEvent, KeyEvent}, keyboard::{PhysicalKey, KeyCode}};
use egui_dnd;


use super::game_folder_structure::GameData;

pub struct ApplicationState{
    screen_state: ScreenState,
    pub interacting_with_ui: bool,
    cursor_inside:bool,
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
    pub fn new() -> Self {
        Self {
            screen_state: ScreenState::MainMenu {
                opened_file: None,
                open_file_dialog: None,
                game_data: None,
                create_new:false,
            },
            cursor_inside:false,
            interacting_with_ui: true,
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
                                    game_data.levels_data.insert(format!("new_level_{}",num+1), LevelData::new());
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
                        //Wwaaaaaaaaaaaa
                    }
                }
            },
        }
    }
    pub fn input_device(&mut self, event: &DeviceEvent, level_state:&mut LevelState){
        match event {
            DeviceEvent::MouseMotion { delta }=>{
                if !self.interacting_with_ui && self.cursor_inside{
                    level_state.camera_controler.process_mouse(delta.0, delta.1);
                }
            },
            _ =>{},
        }
    }
    pub fn input_window(&mut self, event: &WindowEvent, level_state:&mut LevelState, is_event_captured:bool) {
        //Load new level into level_sate
        {
            if let ApplicationState{screen_state:ScreenState::Editor { editor_state,game_data,..},..} = self{
                if let EditorState::LevelSelection { selected_level:Some(selected_level),.. } = editor_state.clone() {
                    *editor_state = EditorState::LevelEditing { selected_level: selected_level.clone() };
                    *level_state = LevelState::from_level_data(&game_data.levels_data[&selected_level]);
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
                        level_state.camera_controler.process_keybord(key_code, state);
                    }
                    match key_code {
                        KeyCode::Escape=>{
                            self.interacting_with_ui = true;
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
}
