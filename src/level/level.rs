use std::f32::consts::PI;

use crate::{
    camer_control::CameraController,
    renderer::{camera::Camera, texture::TextureData},
};

use cgmath::{Point3, Rad, Vector2};
use serde::{Deserialize, Serialize};

use super::{
    hallway::{HallWay, HallWayTexData},
    mesh::{Mesh, MeshTex, Meshable},
    room::{Door, Room},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub start_camera: CameraController,
    pub hallways: Vec<HallWay>,
    pub rooms: Vec<Room>,
}

impl LevelData {
    pub fn none()->Self{
        Self { start_camera: CameraController::new(0., 0., Camera::new([0.,0.,0.],Rad(0.), Rad(0.))), hallways: vec![], rooms: vec![] }
    }
    pub fn new(default_tex_id: &TextureData) -> Self {
        let defualt_mesh_tex = MeshTex::new(default_tex_id.clone(),super::mesh::TileStyle::TileScale(0.1, true));
        let mut rooms = vec![
            Room::new(
                [0., 0., 0.].into(),
                Rad(PI / 4.),
                5.,
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
            ),
            Room::new(
                [20., 0., 0.].into(),
                Rad(-PI/3.),
                3.,
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
            ),
        ];
        let id1 = rooms[0].new_door(Door {
            wall: 0,
            offset: Vector2::new(0., 0.),
            size: Vector2::new(1., 4.),
            vertical_alignment: super::room::VerticalAlign::Bottom,
            horizontal_alignment: super::room::HorizontalAlign::Center,
        });
        let id2 = rooms[1].new_door(Door {
            wall: 3,
            offset: Vector2::new(0., 0.),
            size: Vector2::new(1., 2.),
            vertical_alignment: super::room::VerticalAlign::Bottom,
            horizontal_alignment: super::room::HorizontalAlign::Center,
        });
        let hallway = HallWay::new(
            rooms[0].get_control_rect(&id1, true),
            rooms[1].get_control_rect(&id2, false),
            HallWayTexData::all(defualt_mesh_tex.clone()),
        );
        Self {
            start_camera: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Rad(0.0), Rad(0.0)),
            ),
            hallways: vec![hallway],
            rooms,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelState {
    pub camera_controler: CameraController,
    pub hallways: Vec<HallWay>,
    pub rooms: Vec<Room>,
}

impl LevelState {
    pub fn from_level_data(data: &LevelData) -> Self {
        Self {
            camera_controler: data.start_camera.clone(),
            hallways: data.hallways.clone(),
            rooms: data.rooms.clone(),
        }
    }
}

impl Meshable for LevelState {
    fn mesh(&self) -> Vec<Mesh> {
        let mut meshes = vec![];
        let mut rooms = self.rooms.iter().fold(vec![], |mut acc, room| {
            acc.append(&mut room.mesh());
            acc
        });
        //self.hallways.iter_mut().for_each(|hallway|{hallway.update_door_location(&self.rooms)});
        let mut hallways = self.hallways.iter().fold(vec![], |mut acc, hallway| {
            acc.append(&mut hallway.mesh());
            acc
        });
        meshes.append(&mut rooms);
        meshes.append(&mut hallways);
        meshes
    }
}
