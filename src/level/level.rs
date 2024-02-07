use crate::{
    camer_control::CameraController,
    renderer::{camera::Camera, texture::TextureData},
};

use cgmath::{Point3, Rad, Vector2, Vector3};
use serde::{Deserialize, Serialize};

use super::{
    hallway::{DoorLocation, HallWay, HallWayTexData},
    mesh::{Mesh, MeshTex, Meshable},
    room::{Door, Room, Wall},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub start_camera: CameraController,
    pub hallways: Vec<HallWay>,
    pub rooms: Vec<Room>,
}

impl LevelData {
    pub fn none() -> Self {
        Self {
            start_camera: CameraController::new(
                0.,
                0.,
                Camera::new([0., 0., 0.], Rad(0.), Rad(0.)),
            ),
            hallways: vec![],
            rooms: vec![],
        }
    }
    pub fn new(default_tex_id: &TextureData) -> Self {
        let defualt_mesh_tex = MeshTex::new(
            default_tex_id.clone(),
            super::mesh::TileStyle::TileScale(0.1, true),
        );
        let mut rooms = vec![
            Room::new(
                [0., 0., 0.].into(),
                Rad(0.1),
                5.,
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
                defualt_mesh_tex.clone(),
            ),
            Room::new(
                [0., 0., 20.].into(),
                Rad(0.),
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
            vertical_alignment: super::room::VerticalAlign::Bottom,
            horizontal_alignment: super::room::HorizontalAlign::Center,
        });
        let id2 = rooms[1].new_door(Door {
            wall: 0,
            offset: Vector2::new(0., 0.),
            size: Vector2::new(1., 2.),
            vertical_alignment: super::room::VerticalAlign::Bottom,
            horizontal_alignment: super::room::HorizontalAlign::Center,
        });
        let mut hallway = HallWay::new(
            rooms[0].get_control_rect(&id1, true),
            rooms[1].get_control_rect(&id2, false),
            HallWayTexData::all(defualt_mesh_tex.clone()),
        );
        hallway.start_location = Some(DoorLocation {
            room_index: 0,
            door_id: id1,
        });
        hallway.end_location = Some(DoorLocation {
            room_index: 1,
            door_id: id2,
        });
        rooms[0].moddifiers.push(super::room::Modifier::Ramp {
            pos: Vector3::new(0., 2., 0.),
            dir: Rad(0.2),
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
