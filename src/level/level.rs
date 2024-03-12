use crate::{
    camer_control::CameraController,
    renderer::{camera::Camera, texture::TextureData},
};
use std::collections::HashMap;
use cgmath::{Point3, Deg, Vector2};
use serde::{Deserialize, Serialize};

use super::{
    hallway::HallWay,
    mesh::{Mesh, MeshTex, Meshable},
    room::{Room, RoomId, Wall},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelState {
    pub camera_controler: CameraController,
    pub hallways: Vec<HallWay>,
    pub rooms: HashMap<RoomId,Room>,
}

impl LevelState {
    pub fn update(&mut self){
        self.hallways.iter_mut().for_each(|hallway|{hallway.update_door_location(&self.rooms)});
    }
    pub fn none() -> Self {
        Self {
            camera_controler: CameraController::new(
                0.,
                0.,
                Camera::new([0., 0., 0.], Deg(0.), Deg(0.)),
            ),
            hallways: vec![],
            rooms: HashMap::new(),
        }
    }
    pub fn new(default_tex_id: &TextureData) -> Self {
        let defualt_mesh_tex = MeshTex::new(
            default_tex_id.clone(),
            super::mesh::TileStyle::tile_scale(1., true),
        );
        let mut rooms = vec![
            Room::new(
                "default_room".into(),
                [0., 0., 0.].into(),
                Deg(0.1),
                5.,
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
            ),
        ];
        rooms[0].walls = vec![
            Wall {
                local_pos: Vector2::new(-5., 5.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(-5., -5.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(-4., -6.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(4., -6.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(5., -5.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(5., 5.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(4., 6.),
                wall_texture: defualt_mesh_tex.clone(),
            },
            Wall {
                local_pos: Vector2::new(-4., 6.),
                wall_texture: defualt_mesh_tex.clone(),
            },
        ];
        let mut actual_rooms = HashMap::new();
        actual_rooms.insert(RoomId::new(), rooms.remove(0));
        Self {
            camera_controler: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Deg(0.0), Deg(0.0)),
            ),
            hallways: vec![],
            rooms:actual_rooms,
        }
    }
}

impl Meshable for LevelState {
    fn mesh(&self) -> Vec<Mesh> {
        let mut meshes = vec![];
        let mut rooms = self.rooms.iter().fold(vec![], |mut acc, room| {
            acc.append(&mut room.1.mesh());
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
