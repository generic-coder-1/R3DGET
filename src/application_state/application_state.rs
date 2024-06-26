use crate::{camer_control, level::{hallway::{ControlRect, HallWay, HallWayTexData}, level::LevelState, mesh::{Mesh, MeshTex, Meshable, TileStyle}, room::{Door, DoorId, HorizontalAlign, Modifier, Room, RoomId, VerticalAlign, Wall}}, more_stolen_code::FileDialog, renderer::{self, camera::Camera, texture::{TextureData, TextureId}}, stolen_code_to_update_dependencies};
use egui::{emath, vec2, Button, CollapsingHeader, Color32, ComboBox, Context, DragValue, FontFamily, FontId, FullOutput, Grid, ImageSource, RichText, ScrollArea, Sense, Ui, Vec2, WidgetText};
use egui_modal::Modal;
use instant::Instant;
use itertools::Itertools;
use std::{borrow::Cow, cmp::Ordering, collections::HashMap, path::PathBuf};
use std::hash::Hash;
use winit::{event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};
use egui_dnd::{self};
use cgmath::{Deg, InnerSpace, Point3, Vector2, Vector3};
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
use crate::ModuloSignedExt;
use super::{borrowed_toggle_switch::{self, toggle_ui}, game_folder_structure::GameData};

pub struct ApplicationState{
    screen_state: ScreenState,
    interacting_with_ui: bool,
    cursor_inside:bool,
    default_tex:TextureData,
    render_state:State,
    level_state:LevelState,
    last_render_time:Instant,
    platform:Platform,
}


enum ScreenState {
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
enum EditorState{
    LevelSelection{
        possible_new_level_names:HashMap<String,String>,
        selected_level:Option<String>,
    },
    LevelEditing{
        selected_level:String,
        selected_item:Option<SelectedItem>,
        new_moddifer:Modifier
    },
}


#[derive(Clone)]
enum SelectedItem{
    Room{
        index:RoomId,
    },
    Modifer{
        room_index:RoomId,
        modifer_index:usize,
    },
    Door{
        room_index:RoomId,
        door_id:DoorId,
    },
    HallWay{
        hallway_index:usize,
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
        let mut level = LevelState::new(&default_tex);
        level.camera_controler = camer_control::CameraController::new(
            4.0,
            0.4,
            Camera::new(Point3::new(0.0, 0.0, -10.0), Deg(0.0), Deg(0.0)),
        );
        let level_state = level.clone();
    
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
    pub fn ui(&mut self, ctx: &Context) ->FullOutput{
        let mut screen_state_callbacks:Vec<Box<dyn FnOnce(&mut ScreenState)>> = vec![];
        if let ScreenState::MainMenu { opened_file:Some(folder_path), game_data:Some(game_data),.. } = &self.screen_state{
            game_data.textures.iter().for_each(|(name,data,_)|{
                self.render_state.textures.insert(name.clone(), self.render_state.create_texture(data.clone()));
            });
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
                                    game_data.levels_data.insert(format!("new_level_{}",num+1), LevelState::new(&self.default_tex));
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
                        selected_item,
                        new_moddifer
                    }=>{
                        let level = &mut self.level_state;
                        egui::TopBottomPanel::top("tool bar").show_animated(ctx, self.interacting_with_ui, |ui|{
                            ui.horizontal(|ui|{
                                let mut add_button = |text:&str|{
                                    ui.add(Button::new(text).frame(false))
                                };
                                if add_button("Main Menu").clicked(){
                                    screen_state_callbacks.push(Box::new(|screen_state|{
                                        *screen_state = ScreenState::MainMenu { opened_file: None, open_file_dialog: None, game_data: None, create_new: false }
                                    }));
                                }        
                                if add_button("Level Select").clicked(){
                                    screen_state_callbacks.push(Box::new(|screen_state|{
                                        if let ScreenState::Editor { editor_state, game_data, .. } = screen_state{
                                            *editor_state = EditorState::LevelSelection { possible_new_level_names: game_data.levels.iter().map(|level_name|(level_name.clone(),level_name.clone())).collect(), selected_level: None };
                                        }
                                    }));
                                }                        
                                if add_button("Save").clicked(){
                                    let level_state = level.clone();
                                    let temp: String = selected_level.clone();
                                    screen_state_callbacks.push(Box::new(move |screen_state|{
                                        if let ScreenState::Editor { game_data, folder_path, .. }  = screen_state{
                                            *game_data.levels_data.get_mut(&temp).expect("not possible") = level_state;
                                            game_data.update_folder(folder_path).expect("failed to save");
                                        }
                                    }));
                                }
                            });
                        });
                        fn add_or_delete<T,U>(ui:&mut Ui, iter:&mut HashMap<U,T>, mut callback:impl FnMut(&mut Ui,&U,&mut T), default:T,order:impl FnMut(&(&U,&mut T),&(&U,&mut T))->Ordering)where U:Default + Hash + Eq + Clone{
                            let mut to_add = false;
                            let mut to_delete: Option<U> = None;
                            iter.iter_mut().sorted_by(order).for_each(|(i,value)|{
                                ui.horizontal_top(|ui|{                                    
                                    callback(ui,i,value);
                                    if ui.button("−").clicked(){
                                        to_delete = Some(i.clone());
                                    }
                                });
                            });
                            if ui.button("+").clicked(){                                        
                                to_add = true;
                            }
                            if let Some(i) = to_delete{
                                iter.remove(&i);
                            }
                            if to_add{
                                let mut temp = U::default();
                                while iter.contains_key(&temp){
                                    temp = U::default();
                                }
                                iter.insert(temp, default);
                            }
                        }
                        fn add_or_delete2<T>(ui:&mut Ui, iter:&mut Vec<T>, mut callback:impl FnMut(&mut Ui,usize,&T), default:&T)where T:Clone{
                            let mut to_add: bool = false;
                            let mut to_delete: Option<usize> = None;
                            iter.iter().enumerate().for_each(|(i,value)|{
                                ui.horizontal(|ui|{                                 
                                    callback(ui,i,value);
                                    if ui.button("−").clicked(){
                                        to_delete = Some(i);
                                    }
                                });
                            });
                            if ui.button("+").clicked(){
                                to_add = true;
                            }
                            if let Some(i) = to_delete{
                                iter.remove(i);
                            }
                            if to_add{
                                iter.push(default.clone());
                            }
                        }
                        egui::SidePanel::left("selector").show_animated(ctx,self.interacting_with_ui, |ui|{
                            egui::ScrollArea::new([false,true]).show(ui, |ui|{
                                CollapsingHeader::new(RichText::new("Rooms").heading()).default_open(true).show(ui, |ui|{
                                    let default_tex = MeshTex::new(TextureData::new(&self.render_state.default_texture, "default".into()),TileStyle::tile_scale(1., true));
                                    let room_callback = |ui:&mut Ui,i: &RoomId,room: &mut Room|{
                                        ui.collapsing(format!("Room: {}",&room.name), |ui|{
                                            if ui.label("Room").clicked(){
                                                let i2 = i.clone();
                                                screen_state_callbacks.push(Box::new(move |screen_state|{
                                                    if let ScreenState::Editor { editor_state:EditorState::LevelEditing {selected_item, .. } , .. } = screen_state{
                                                        *selected_item = Some(SelectedItem::Room { index: i2 });
                                                    };
                                                }));
                                            }
                                            ui.collapsing("Modifers", |ui|{
                                                ComboBox::from_label("New Moddifer")
                                                    .selected_text(match &new_moddifer{
                                                        crate::level::room::Modifier::Ramp { .. } => "Ramp",
                                                        crate::level::room::Modifier::Cliff { .. } => "Extend",
                                                        crate::level::room::Modifier::Disc { .. } => "Platform",
                                                    })
                                                    .show_ui(ui, |ui|{
                                                    ui.selectable_value(new_moddifer, Modifier::Disc { pos: Vector3::new(0., 0., 0.), size: Vector3::new(1., 1., 1.), sides: vec![default_tex.clone(),default_tex.clone(),default_tex.clone(),default_tex.clone(),default_tex.clone()], dir: Deg(0.), top_tex: default_tex.clone(), bottom_tex: default_tex.clone() }, "Platform");
                                                    ui.selectable_value(new_moddifer, Modifier::Ramp { pos: Vector3::new(0., 0., 0.), size: Vector3::new(1., 1., 1.), ramp_texture: default_tex.clone(),dir:Deg(0.), wall_texture: default_tex.clone(), bottom_texture: default_tex.clone() },"Ramp");
                                                    ui.selectable_value(new_moddifer, Modifier::Cliff {walls: vec![Wall {local_pos: Vector2::new(-1., -1.),wall_texture: default_tex.clone(),},Wall {local_pos: Vector2::new(1., -1.),wall_texture: default_tex.clone(),},Wall {local_pos: Vector2::new(1., 1.),wall_texture: default_tex.clone(),},Wall {local_pos: Vector2::new(-1., 1.),wall_texture: default_tex.clone(),},],on_roof: false,height: 1.,floor_texture: default_tex.clone(),}, "Extend");
                                                });
                                                let moddifer_callback = |ui: &mut Ui,j,moddifier:&Modifier|{
                                                    if ui.label(match &moddifier{
                                                        crate::level::room::Modifier::Ramp { .. } => "Ramp",
                                                        crate::level::room::Modifier::Cliff { .. } => "Extend",
                                                        crate::level::room::Modifier::Disc { .. } => "Platform",
                                                    }).clicked(){
                                                        let i2 = i.clone();
                                                        screen_state_callbacks.push(Box::new(move |screen_state|{
                                                            if let ScreenState::Editor { editor_state:EditorState::LevelEditing {selected_item, .. } , .. } = screen_state{
                                                                *selected_item = Some(SelectedItem::Modifer { room_index: i2,modifer_index:j });
                                                            }
                                                        }));
                                                    };
                                                };
                                                add_or_delete2(ui, &mut room.moddifiers, moddifer_callback,& new_moddifer);
                                            });
                                            ui.collapsing("Doors", |ui|{
                                                let door_callback = |ui:&mut Ui,id:&DoorId,_door:&mut Door|{
                                                    if ui.label(format!("Id:{}",id.0.get())).clicked(){
                                                        let a =id.clone();
                                                        let i2 = i.clone();
                                                        screen_state_callbacks.push(Box::new(move |screen_state|{
                                                            if let ScreenState::Editor { editor_state:EditorState::LevelEditing {selected_item, .. } , .. } = screen_state{
                                                                *selected_item = Some(SelectedItem::Door { room_index: i2, door_id: a });
                                                            }
                                                        }));
                                                    };
                                                };
                                                add_or_delete(ui, &mut room.doors, door_callback, Door { wall: 0, offset: Vector2::new(0., 0.), size: Vector2::new(1., 3.), center: (VerticalAlign::Bottom,HorizontalAlign::Center) },|a,b|{a.0.cmp(b.0)});
                                            });
                                        });
                                    };
                                    let new_name = level.rooms.iter().fold(1, |acc, room|{if room.1.name.starts_with("New Room") {acc+1}else{acc}});
                                    add_or_delete(ui, &mut level.rooms, room_callback, Room::new(format!("New Room {}",new_name), Vector3::new(0., 0., 0.), Deg(0.), 5., default_tex.clone(), default_tex.clone(), default_tex.clone()),|a,b|{a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())});
                                });
                                CollapsingHeader::new(RichText::new("Hallways").heading()).default_open(true).show(ui,|ui|{
                                    let hallway_callback = |ui:&mut Ui,i: usize,_hallway: &HallWay|{
                                        if ui.label(format!("Hallway {}",i+1)).clicked(){
                                            screen_state_callbacks.push(Box::new(move |screen_state|{
                                                if let ScreenState::Editor { editor_state:EditorState::LevelEditing {selected_item, .. } , .. } = screen_state{
                                                    *selected_item = Some(SelectedItem::HallWay { hallway_index: i });
                                                };
                                            }));
                                        };
                                    };
                                    add_or_delete2(ui, &mut level.hallways, hallway_callback, &HallWay::new(ControlRect::new(Vector3::new(0., 0., 0.), Deg(0.), Vector2::new(1.,3.)), ControlRect::new(Vector3::new(0., 0., 0.), Deg(0.), Vector2::new(1.,3.)), HallWayTexData::all(MeshTex::new(self.default_tex.clone(), TileStyle::tile_scale(1., true)))))

                                })
                            });
                        });         
                        let get_egui_image_sorce = |texture_id:&TextureId|->ImageSource{
                            if let Some((name,data,exetension)) = game_data.textures.iter().find(|a|a.0 == *texture_id){
                                ImageSource::Bytes { uri: Cow::Owned(format!("bytes://{}.{}",name,exetension)), bytes: egui::load::Bytes::Shared(data.clone()) }
                            }else{
                                egui::include_image!("..\\renderer\\default.png")
                            }
                        };
                        let default_tex = self.default_tex.clone();
                        let add_texture_controls = |ui:&mut Ui,name:&str,texture:&mut MeshTex|{
                            ui.collapsing(name,|ui|{
                                ui.menu_button(format!("Id: {}",texture.id.id), |ui|{
                                    egui::Grid::new("texture selection grid").show(ui, |ui|{
                                        let mut i = 1;
                                        ui.vertical(|ui|{
                                            let respose = ui.add(egui::Image::new(get_egui_image_sorce(&"Default".into())).sense(Sense::click().union(Sense::hover())).max_width(20.));
                                                    if respose.clicked(){
                                                        texture.id = default_tex.clone();
                                                        ui.close_menu();
                                                    }
                                                    respose.on_hover_text("default");
                                        });
                                        for (name,_,_) in game_data.textures.iter(){
                                            ui.vertical(|ui|{
                                                ui.allocate_ui(Vec2::new(100., 100.), |ui|{
                                                    let respose = ui.add(egui::Image::new(get_egui_image_sorce(&name)).sense(Sense::click().union(Sense::hover())).max_width(20.));
                                                    if respose.clicked(){
                                                        texture.id = TextureData::new(self.render_state.textures.get(name).unwrap(), name.clone());
                                                        ui.close_menu();
                                                    }
                                                    respose.on_hover_text(name.as_ref());
                                                });
                                            });
                                            i+=1;
                                            if i>2{
                                                i=0;
                                                ui.end_row()
                                            }
                                        }
                                    });
                                    
                                });
                                ui.collapsing("Offset", |ui|{
                                    add_drag_value(ui, "X:", &mut texture.offset[0], 0.05);
                                    add_drag_value(ui, "Y:", &mut texture.offset[1], 0.05);
                                });
                                ui.collapsing("Fliped", |ui|{
                                    ui.horizontal(|ui|{
                                        ui.label("X:");
                                        borrowed_toggle_switch::toggle_ui(ui, &mut texture.fliped[0]);
                                    });
                                    ui.horizontal(|ui|{
                                        ui.label("Y:");
                                        borrowed_toggle_switch::toggle_ui(ui, &mut texture.fliped[1]);
                                    });
                                });
                                ui.collapsing("Tile Mode", |ui|{
                                    ui.horizontal(|ui|{
                                        ui.label("Scale");
                                        toggle_ui(ui, &mut texture.tile.is_specific);
                                        ui.label("Specfifc");
                                    });
                                    match &mut texture.tile.is_specific {
                                        true=>{
                                            let crate::level::mesh::TileSpecific(x,y)=&mut texture.tile.specific;
                                            add_drag_value(ui, "X:", x, 0.1);
                                            add_drag_value(ui, "Y:", y, 0.1);
                                        }
                                        false=>{
                                            let crate::level::mesh::TileScale(scale,global)=&mut texture.tile.scale;
                                            add_drag_value(ui, "Scale:", scale, 0.1);
                                            ui.horizontal(|ui|{
                                                ui.label("Global:");
                                                toggle_ui(ui, global);
                                            });
                                        },
                                    }
                                });
                                ui.add(egui::Image::new(get_egui_image_sorce(&texture.id.id)).max_width(100.));
                            });
                        };
                        egui::SidePanel::right("editor").resizable(true).show_animated(ctx, self.interacting_with_ui, |ui|{
                            ScrollArea::new([false,true]).show(ui, |ui|{
                                if let Some(selected_item) = selected_item{
                                    match selected_item{
                                        SelectedItem::Room { index } => {
                                            if let Some(room) = level.rooms.get_mut(&index){                                                
                                                ui.horizontal(|ui|{
                                                    ui.add(egui::Label::new("Name:").wrap(false));
                                                    ui.text_edit_singleline(&mut room.name);
                                                });
                                                ui.collapsing("Position", |ui|{                                                
                                                    add_drag_value(ui,"X:",&mut room.position.x,0.1);
                                                    add_drag_value(ui,"Y:",&mut room.position.y,0.1);
                                                    add_drag_value(ui,"Z:",&mut room.position.z,0.1);
                                                    add_drag_value(ui,"Rot:",&mut room.rotation.0,1.);
                                                    add_drag_value(ui,"Height",&mut room.height, 0.1);
                                                });
                                                ui.collapsing("Textures", |ui|{
                                                    add_texture_controls(ui, "Floor texture",&mut room.floor_texture);
                                                    add_texture_controls(ui, "Roof texture",&mut room.roof_texture);
                                                });
                                                ui.collapsing("Walls", |ui|{
                                                    let mut wall_to_remove=None;
                                                    let mut wall_to_add=None;
                                                    (0..room.walls.len()).into_iter().for_each(|i|{
                                                        let wall = &mut room.walls[i];
                                                        ui.collapsing(format!("Wall {i}"), |ui|{
                                                            add_drag_value(ui, "X:", &mut wall.local_pos.x, 0.1);
                                                            add_drag_value(ui, "Y:", &mut wall.local_pos.y, 0.1);
                                                            add_texture_controls(ui,"Texture",&mut wall.wall_texture);
                                                            ui.horizontal(|ui|{  
                                                                if ui.button("−").clicked(){
                                                                    wall_to_remove = Some(i);
                                                                };
                                                                if ui.button("+").clicked(){
                                                                    wall_to_add = Some(i+1);
                                                                };
                                                            });
                                                        });
                                                    });
                                                    if let Some(i) = wall_to_remove{
                                                        room.walls.remove(i);
                                                    }
                                                    if let Some(i) = wall_to_add{
                                                        room.walls.insert(i,Wall::new((room.walls[((i as isize -1)%(room.walls.len() as isize)) as usize].local_pos + room.walls[((i as isize)%(room.walls.len() as isize)) as usize].local_pos)/2., room.walls[((i as isize -1)%(room.walls.len() as isize)) as usize].wall_texture.clone()));
                                                    }
                                                });
                                            }
                                        },
                                        SelectedItem::Modifer { room_index, modifer_index } => {
                                            if let Some(modifer) = level.rooms.get_mut(&room_index).and_then(|room|{room.moddifiers.get_mut(*modifer_index)}){                                                
                                                match modifer{
                                                    crate::level::room::Modifier::Ramp { pos, dir, size, ramp_texture, wall_texture, bottom_texture } => {
                                                        ui.collapsing("Position", |ui|{                                                        
                                                            add_drag_value(ui, "X:", &mut pos.x, 0.1);
                                                            add_drag_value(ui, "Y:", &mut pos.y, 0.1);
                                                            add_drag_value(ui, "Z:", &mut pos.z, 0.1);
                                                            add_drag_value(ui, "Rot:", &mut dir.0, 1.0);
                                                        });
                                                        ui.collapsing("Size", |ui|{
                                                            add_drag_value(ui, "X:", &mut size.x, 0.1);
                                                            add_drag_value(ui, "Y:", &mut size.y, 0.1);
                                                            add_drag_value(ui, "Z:", &mut size.z, 0.1);
                                                        });
                                                        ui.collapsing("Textures", |ui|{
                                                            add_texture_controls(ui,"Ramp Texture",ramp_texture);
                                                            add_texture_controls(ui,"Wall Texture",wall_texture);
                                                            add_texture_controls(ui,"Bottom Texture",bottom_texture);
    
                                                        });                                                    
                                                    },
                                                    crate::level::room::Modifier::Cliff { walls, on_roof, height, floor_texture } => {
                                                        toggle_ui(ui, on_roof);
                                                        add_drag_value(ui, "Height:", height, 0.1);
                                                        add_texture_controls(ui,"Floor Texture",floor_texture);
                                                        ui.collapsing("Walls", |ui|{
                                                            let mut wall_to_remove=None;
                                                            let mut wall_to_add=None;
                                                            (0..walls.len()).into_iter().for_each(|i|{
                                                                let wall = &mut walls[i];
                                                                ui.collapsing(format!("Wall {i}"), |ui|{
                                                                    add_drag_value(ui, "X:", &mut wall.local_pos.x, 0.1);
                                                                    add_drag_value(ui, "Y:", &mut wall.local_pos.y, 0.1);
                                                                    add_texture_controls(ui,"Texture",&mut wall.wall_texture);
                                                                    ui.horizontal(|ui|{  
                                                                        if ui.button("−").clicked(){
                                                                            wall_to_remove = Some(i);
                                                                        };
                                                                        if ui.button("+").clicked(){
                                                                            wall_to_add = Some(i+1);
                                                                        };
                                                                    });
                                                                });
                                                            });
                                                            if let Some(i) = wall_to_remove{
                                                                walls.remove(i);
                                                            }
                                                            if let Some(i) = wall_to_add{
                                                                walls.insert(i,Wall::new((walls[((i as isize -1)%(walls.len() as isize)) as usize].local_pos + walls[((i as isize)%(walls.len() as isize)) as usize].local_pos)/2., walls[((i as isize -1)%(walls.len() as isize)) as usize].wall_texture.clone()));
                                                            }
                                                        });
                                                    },
                                                    crate::level::room::Modifier::Disc { pos, size, sides, dir, top_tex, bottom_tex } => {
                                                        ui.collapsing("Position", |ui|{                                                        
                                                            add_drag_value(ui, "X:", &mut pos.x, 0.1);
                                                            add_drag_value(ui, "Y:", &mut pos.y, 0.1);
                                                            add_drag_value(ui, "Z:", &mut pos.z, 0.1);
                                                            add_drag_value(ui, "Rot:", &mut dir.0, 1.0);
                                                        });
                                                        ui.collapsing("Size", |ui|{
                                                            add_drag_value(ui, "X:", &mut size.x, 0.1);
                                                            add_drag_value(ui, "Y:", &mut size.y, 0.1);
                                                            add_drag_value(ui, "Z:", &mut size.z, 0.1);
                                                        });
                                                        ui.collapsing("Texture", |ui|{
                                                            add_texture_controls(ui,"Top Texture",top_tex);
                                                            add_texture_controls(ui,"Bottom Texture",bottom_tex);
                                                        });
                                                        ui.collapsing("Sides", |ui|{
                                                            let mut side_to_remove = None;
                                                            let mut side_to_add = None;
                                                            sides.iter_mut().enumerate().for_each(|(i,side_tex)|{
                                                                ui.collapsing(format!("Side {}",i), |ui|{
                                                                    add_texture_controls(ui,"Texture",side_tex);
                                                                    ui.horizontal(|ui|{  
                                                                        if ui.button("−").clicked(){
                                                                            side_to_remove = Some(i);
                                                                        };
                                                                        if ui.button("+").clicked(){
                                                                            side_to_add = Some(i+1);
                                                                        };
                                                                    });
                                                                });
                                                            });
                                                            if let Some(i) = side_to_remove{
                                                                sides.remove(i);
                                                            }
                                                            if let Some(i) = side_to_add{
                                                                sides.insert(i, sides[i-1].clone());
                                                            }
                                                        });
                                                    },
                                                }
                                            }
                                        },
                                        SelectedItem::Door { room_index, door_id } => {
                                            if let (Some(num_walls),Some(door)) = (level.rooms.get(&room_index).and_then(|room|{Some(room.walls.len())}),level.rooms.get_mut(&room_index).and_then(|room|{room.doors.get_mut(&door_id)})){
                                                ui.collapsing("Position", |ui|{
                                                    add_drag_value(ui, "Wall", &mut door.wall, 0.1);                                            
                                                    Grid::new("center").num_columns(3).min_col_width(10.).min_row_height(10.).spacing(Vec2::new(1., 1.)).show(ui, |ui|{
                                                        let mut selectable_value2 = |ui:&mut Ui,a|{
                                                            let mut button = Button::new("").min_size([20.,20.].into());
                                                            if door.center == a{
                                                                button = button.selected(true);
                                                            }
                                                            if ui.add(button).clicked(){
                                                                door.center = a;
                                                            }
                                                        };
                                                        selectable_value2(ui,(VerticalAlign::Top,HorizontalAlign::Left));
                                                        selectable_value2(ui,(VerticalAlign::Top,HorizontalAlign::Center));
                                                        selectable_value2(ui,(VerticalAlign::Top,HorizontalAlign::Right));
                                                        ui.end_row();
                                                        selectable_value2(ui,(VerticalAlign::Center,HorizontalAlign::Left));
                                                        selectable_value2(ui,(VerticalAlign::Center,HorizontalAlign::Center));
                                                        selectable_value2(ui,(VerticalAlign::Center,HorizontalAlign::Right));
                                                        ui.end_row();
                                                        selectable_value2(ui,(VerticalAlign::Bottom,HorizontalAlign::Left));
                                                        selectable_value2(ui,(VerticalAlign::Bottom,HorizontalAlign::Center));
                                                        selectable_value2(ui,(VerticalAlign::Bottom,HorizontalAlign::Right));
                                                        ui.end_row();
                                                    });
                                                    add_drag_value(ui, "X:", &mut door.offset.x, 0.01);
                                                    add_drag_value(ui, "Y:", &mut door.offset.y, 0.01);
                                                });
                                                ui.collapsing("Size", |ui|{
                                                    add_drag_value(ui, "X:", &mut door.size.x, 0.01);
                                                    add_drag_value(ui, "Y:", &mut door.size.y, 0.01);
                                                });
                                                door.wall = door.wall.modulo(num_walls as isize);
                                            }
                                        },
                                        SelectedItem::HallWay { hallway_index } => {
                                            fn add_control_rect_controls(ui:&mut Ui,name:impl Into<WidgetText>,control_rect:&mut ControlRect){
                                                ui.collapsing(name, |ui|{
                                                    ui.collapsing("Position", |ui|{
                                                        add_drag_value(ui, "X:", &mut control_rect.position.x, 0.1);
                                                        add_drag_value(ui, "Y:", &mut control_rect.position.y, 0.1);
                                                        add_drag_value(ui, "Z:", &mut control_rect.position.z, 0.1);
                                                    });
                                                    add_drag_value(ui, "Rot:", &mut control_rect.rotation.0, 1.0);
                                                    ui.collapsing("Size", |ui|{
                                                        add_drag_value(ui,"X:", &mut control_rect.size.x, 0.01);
                                                        add_drag_value(ui,"Y:", &mut control_rect.size.y, 0.01);
                                                    });
                                                });
                                            }
                                            if let Some(hallway) = level.hallways.get_mut(*hallway_index){                                                
                                                let add_hallway_texture_controls = |ui:&mut Ui,hallway_texs:&mut HallWayTexData|{
                                                    ui.collapsing("Textures", |ui|{
                                                        add_texture_controls(ui,"Top",&mut hallway_texs.top);
                                                        add_texture_controls(ui,"Bottom",&mut hallway_texs.bottom);
                                                        add_texture_controls(ui,"Left",&mut hallway_texs.left);
                                                        add_texture_controls(ui,"Right",&mut hallway_texs.right);
                                                    });
                                                };
                                                ui.collapsing("Start", |ui|{
                                                    ui.horizontal(|ui|{
                                                        ui.label("Snap to door");
                                                        toggle_ui(ui, &mut hallway.start_location.enabled);
                                                    });
                                                    CollapsingHeader::new("Starting Location").enabled(hallway.start_location.enabled).show(ui,|ui|{
                                                        ComboBox::from_label("Room").selected_text(
                                                            if let Some(room_id) = hallway.start_location.room_index{
                                                                if let Some(room) = level.rooms.get(&room_id){
                                                                    room.name.clone()
                                                                }else{
                                                                    "None".into()
                                                                }
                                                            }else{
                                                                "None".into()
                                                            })
                                                            .show_ui(ui, |ui|{
                                                                ui.selectable_value(&mut hallway.start_location.room_index, None, "None");
                                                                level.rooms.iter().sorted_by(|a,b|a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())).for_each(|(i,value)|{
                                                                ui.selectable_value(&mut hallway.start_location.room_index, Some(i.clone()), &value.name);
                                                            });
                                                        });
                                                        ComboBox::from_label("Door").selected_text(
                                                            if let Some(room_id) = hallway.start_location.room_index{
                                                                if let Some(door_id) = hallway.start_location.door_id{
                                                                    if let Some(room) = level.rooms.get(&room_id){                                                                    
                                                                        if room.doors.contains_key(&door_id){
                                                                            format!("{:?}",door_id)
                                                                        }else{
                                                                            "None".into()
                                                                        }
                                                                    }else{
                                                                        "None".into()
                                                                    }
                                                                }else{
                                                                    "None".into()
                                                                }
                                                            }else{
                                                                "None".into()
                                                            }
                                                        ).show_ui(ui, |ui|{
                                                            ui.selectable_value(&mut hallway.start_location.door_id, None, "None");
                                                            if let Some(room_id) = hallway.start_location.room_index{
                                                                if let Some(room) = level.rooms.get(&room_id){
                                                                    room.doors.iter().sorted_by(|a,b|a.0.cmp(&b.0)).for_each(|(i,_)|{
                                                                        ui.selectable_value(&mut hallway.start_location.door_id, Some(i.clone()), i.to_string());
                                                                    });
                                                                }
                                                            }
                                                        });
                                                    });
                                                    ui.add_enabled_ui(!hallway.start_location.enabled, |ui|{
                                                        add_control_rect_controls(ui, "Start", &mut hallway.start);
                                                    });
                                                    add_hallway_texture_controls(ui,&mut hallway.start_texture);
                                                });
                                                ui.collapsing("Middle", |ui|{
                                                    if hallway.middle.len()>0{
                                                        let mut to_add: Option<usize> = None;
                                                        let mut to_delete: Option<usize> = None;
                                                        hallway.middle.iter_mut().enumerate().for_each(|(i,segment)|{
                                                            ui.collapsing(format!("Segment {i}"), |ui|{
                                                                add_control_rect_controls(ui, "Control Rect", &mut segment.0);
                                                                add_hallway_texture_controls(ui,&mut segment.1);
                                                            });
                                                            ui.horizontal(|ui|{                                 
                                                                if ui.button("−").clicked(){
                                                                    to_delete = Some(i);
                                                                }
                                                                if ui.button("+").clicked(){
                                                                    to_add = Some(i+1);
                                                                }
                                                            });
                                                        });
                                                        if let Some(i) = to_delete{
                                                            hallway.middle.remove(i);
                                                        }
                                                        if let Some(i) = to_add{
                                                            let c = &hallway.middle[i-1];
                                                            let b = &hallway.middle.get(i).and_then(|a|{Some(&a.0)}).unwrap_or(&hallway.end);
                                                            let a = if i==0{(&hallway.start,&hallway.start_texture)}else{(&c.0,&c.1)};
                                                            hallway.middle.insert(i, (
                                                                ControlRect{
                                                                    position: (a.0.position + b.position)/2.,
                                                                    rotation: b.position.xz().angle(a.0.position.xz()).into(),
                                                                    size: (a.0.size + b.size)/2.,
                                                                },
                                                                a.1.clone()
                                                            ));
                                                        }
                                                    }else{
                                                        if ui.button("+").clicked(){
                                                            hallway.middle.push((
                                                                ControlRect{
                                                                    position: (hallway.start.position + hallway.end.position)/2.,
                                                                    rotation: hallway.end.position.xz().angle(hallway.start.position.xz()).into(),
                                                                    size: (hallway.start.size + hallway.end.size)/2.,
                                                                },
                                                                hallway.start_texture.clone()
                                                            ));
                                                        }
                                                    }
                                                });
                                                ui.collapsing("End", |ui|{
                                                    ui.horizontal(|ui|{
                                                        ui.label("Snap to door");
                                                        toggle_ui(ui, &mut hallway.end_location.enabled);
                                                    });
                                                    CollapsingHeader::new("Ending Location").enabled(hallway.end_location.enabled).show(ui,|ui|{
                                                        ComboBox::from_label("Room").selected_text(
                                                            if let Some(room_id) = hallway.end_location.room_index{
                                                                if let Some(room) = level.rooms.get(&room_id){
                                                                    room.name.clone()
                                                                }else{
                                                                    "None".into()
                                                                }
                                                            }else{
                                                                "None".into()
                                                            }).show_ui(ui, |ui|{
                                                            ui.selectable_value(&mut hallway.end_location.room_index, None, "None");
                                                            level.rooms.iter().sorted_by(|a,b|a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())).for_each(|(i,value)|{
                                                                ui.selectable_value(&mut hallway.end_location.room_index, Some(i.clone()), &value.name);
                                                            });
                                                        });
                                                        ComboBox::from_label("Door").selected_text(
                                                            if let Some(room_id) = hallway.end_location.room_index{
                                                                if let Some(door_id) = hallway.end_location.door_id{
                                                                    if let Some(room) = level.rooms.get(&room_id){                                                                    
                                                                        if room.doors.contains_key(&door_id){
                                                                            format!("{:?}",door_id.0.get())
                                                                        }else{
                                                                            "None".into()
                                                                        }
                                                                    }else{
                                                                        "None".into()
                                                                    }
                                                                }else{
                                                                    "None".into()
                                                                }
                                                            }else{
                                                                "None".into()
                                                            }
                                                        ).show_ui(ui, |ui|{
                                                            ui.selectable_value(&mut hallway.end_location.door_id, None, "None");
                                                            if let Some(room_id) = hallway.end_location.room_index{
                                                                if let Some(room) = level.rooms.get(&room_id){                                                                
                                                                    room.doors.iter().sorted_by(|a: &(&DoorId, &Door),b|a.0.cmp(&b.0)).for_each(|(i,_)|{
                                                                        ui.selectable_value(&mut hallway.end_location.door_id, Some(i.clone()), i.to_string());
                                                                    });
                                                                }
                                                            }
                                                        });
                                                    });
                                                    ui.add_enabled_ui(!hallway.end_location.enabled, |ui|{
                                                        add_control_rect_controls(ui, "end", &mut hallway.end);
                                                    });
                                                });
                                            }
                                        },
                                    }
                                }
                            });
                        });
                    }
                }
            },
        }
        let return_val = ctx.end_frame();
        screen_state_callbacks.into_iter().for_each(|callback: Box<dyn FnOnce(&mut ScreenState)>|{
            callback(&mut self.screen_state);
        });
        return_val
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
                    let default_tex: MeshTex = MeshTex::new(self.default_tex.clone(), TileStyle::tile_scale(1., true));
                    *editor_state = EditorState::LevelEditing { selected_level: selected_level.clone(),selected_item:None,new_moddifer:Modifier::Disc { pos: Vector3::new(0., 0., 0.), size: Vector3::new(1., 1., 1.), sides: vec![default_tex.clone(),default_tex.clone(),default_tex.clone(),default_tex.clone(),default_tex.clone()], dir: Deg(0.), top_tex: default_tex.clone(), bottom_tex: default_tex.clone() } };
                    self.level_state = game_data.levels_data[&selected_level].clone();
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
                            self.ui(&ctx)
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
                self.set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            });
        }
    }
}

fn add_drag_value<T>(ui:&mut Ui,name:&str, value:&mut T,speed:f64)where T:emath::Numeric{
    ui.horizontal(|ui|{
        ui.label(name);
        ui.add(DragValue::new(value).speed(speed))
    });
}