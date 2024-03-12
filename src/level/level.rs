use crate::{
    camer_control::CameraController,
    renderer::{camera::Camera, texture::TextureData},
};
use std::{collections::HashMap, f32::consts::PI};
use cgmath::{Point3, Deg, Vector2, Vector3};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    hallway::{DoorLocation, HallWay, HallWayTexData},
    mesh::{Mesh, MeshTex, Meshable},
    room::{Door, Room, RoomId, Wall},
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
                "default_room_1".into(),
                [0., 0., 0.].into(),
                Deg(0.1),
                5.,
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
            ),
            Room::new(
                "default_room_2".into(),
                [0., 0., 20.].into(),
                Deg(0.),
                3.,
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
        let id1 = rooms[0].new_door(Door {
            wall: 6,
            offset: Vector2::new(0., 0.),
            size: Vector2::new(1., 4.),
            center:(super::room::VerticalAlign::Bottom,super::room::HorizontalAlign::Center),
        });
        let id2 = rooms[1].new_door(Door {
            wall: 0,
            offset: Vector2::new(0., 0.),
            size: Vector2::new(1., 2.),
            center:(super::room::VerticalAlign::Bottom,super::room::HorizontalAlign::Center),
        });
        let mut hallway = HallWay::new(
            rooms[0].get_control_rect(&id1, true).unwrap(),
            rooms[1].get_control_rect(&id2, false).unwrap(),
            HallWayTexData::all(defualt_mesh_tex.clone()),
        );
        rooms[0].moddifiers.push(super::room::Modifier::Ramp {
            pos: Vector3::new(0., 2., 0.),
            dir: Deg(0.2),
            size: Vector3::new(1., 2., 3.),
            ramp_texture: defualt_mesh_tex.clone(),
            wall_texture: defualt_mesh_tex.clone(),
            bottom_texture: defualt_mesh_tex.clone(),
        });
        rooms[0].moddifiers.push(super::room::Modifier::Cliff {
            walls: vec![
                Wall {
                    local_pos: Vector2::new(-1., -1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(1., -1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(1., 1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(-1., 1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
            ],
            on_roof: false,
            height: 1.,
            floor_texture: defualt_mesh_tex.clone(),
        });
        rooms[0].moddifiers.push(super::room::Modifier::Cliff {
            walls: vec![
                Wall {
                    local_pos: Vector2::new(-3., -3.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(-1., -3.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(-1., -1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
                Wall {
                    local_pos: Vector2::new(-3., -1.),
                    wall_texture: defualt_mesh_tex.clone(),
                },
            ],
            on_roof: true,
            height: 1.,
            floor_texture: defualt_mesh_tex.clone(),
        });
        rooms[0].moddifiers.push(super::room::Modifier::Disc {
            pos: Vector3::new(3., 3., 3.),
            size: Vector3::new(1., 0.5, 2.),
            sides: (0..4).into_iter().map(|_| defualt_mesh_tex.clone()).collect_vec(),
            dir: Deg(PI/4.),
            top_tex: defualt_mesh_tex.clone(),
            bottom_tex: defualt_mesh_tex.clone(),
        });
        let room1_id=RoomId::new();
        let room2_id=RoomId::new();
        hallway.start_location = DoorLocation {
            room_index: Some(room1_id),
            door_id: Some(id1),
            enabled:true,
        };
        hallway.end_location = DoorLocation {
            room_index: Some(room2_id),
            door_id: Some(id2),
            enabled:true
        };
        let room1 = rooms.remove(0);
        let room2 = rooms.remove(0);
        let mut actual_rooms = HashMap::new();
        actual_rooms.insert(room1_id, room1);
        actual_rooms.insert(room2_id, room2);
        Self {
            camera_controler: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Deg(0.0), Deg(0.0)),
            ),
            hallways: vec![hallway],
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
