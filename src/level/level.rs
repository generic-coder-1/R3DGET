use std::collections::HashMap;

use crate::{
    camer_control::CameraController,
    renderer::{camera::Camera, texture::TextureId},
};

use cgmath::{Point3, Rad};
use serde::{Deserialize, Serialize};

use super::{
    control_rect::{ControlRect, ControlRectId},
    hallway::HallWay,
    mesh::{Mesh, MeshTex, Meshable},
    room::Room,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub start_camera: CameraController,
    pub control_rects: HashMap<ControlRectId, ControlRect>,
    pub hallways: Vec<HallWay>,
    pub rooms: Vec<Room>,
}

impl LevelData {
    pub fn new(default_tex_id: &TextureId) -> Self {
        let control_rects = HashMap::new();
        let defualt_mesh_tex = MeshTex::new(default_tex_id.clone());
        Self {
            start_camera: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Rad(0.0), Rad(0.0)),
            ),
            control_rects,
            hallways: vec![],
            rooms: vec![Room::new(
                [0., 0., 0.].into(),
                Rad(0.),
                5.,
                &defualt_mesh_tex,
                &defualt_mesh_tex,
                &defualt_mesh_tex,
            )],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelState {
    pub camera_controler: CameraController,
    pub control_rects: HashMap<ControlRectId, ControlRect>,
    pub hallways: Vec<HallWay>,
    pub rooms: Vec<Room>,
}

impl LevelState {
    pub fn from_level_data(data: &LevelData) -> Self {
        Self {
            camera_controler: data.start_camera.clone(),
            control_rects: data.control_rects.clone(),
            hallways: data.hallways.clone(),
            rooms:data.rooms.clone()
        }
    }
}

impl Meshable for LevelState {
    fn mesh(&self) -> Vec<Mesh> {
        let mut meshes = vec![];
        let mut rooms = self.rooms.iter().fold(vec![], |mut acc,room|{acc.append(&mut room.mesh());acc});
        meshes.append(&mut rooms);
        meshes
    }
}
