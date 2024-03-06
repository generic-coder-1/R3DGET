use std::collections::HashMap;

use crate::level::mesh::MeshTex;

use cgmath::{Array, Basis2, MetricSpace, Deg, Rotation, Rotation2, Vector2, Vector3};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    mesh::{Mesh, MeshVertex, Meshable},
    room::{DoorId, Room, RoomId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallWay {
    pub start: ControlRect,
    pub start_texture: HallWayTexData,
    pub middle: Vec<(ControlRect, HallWayTexData)>,
    pub end: ControlRect,
    pub start_location: DoorLocation,
    pub end_location: DoorLocation,
}

impl HallWay {
    pub fn new(start: ControlRect, end: ControlRect, texture: HallWayTexData) -> Self {
        Self {
            start,
            start_texture: texture,
            middle: vec![],
            end,
            start_location: DoorLocation {
                room_index: None,
                door_id: None,
                enabled: false,
            },
            end_location: DoorLocation {
                room_index: None,
                door_id: None,
                enabled: false,
            },
        }
    }
    pub fn update_door_location(&mut self, rooms: &HashMap<RoomId, Room>) {
        if self.start_location.enabled {
            if let Some(room_index) = self.start_location.room_index {
                if let Some(door_id) = self.start_location.door_id {
                    if let Some(room) = rooms.get(&room_index){                        
                        if let Some(c_rect) = room.get_control_rect(&door_id, false) {
                            self.start = c_rect;
                        }
                    }
                }
            }
        }
        if self.end_location.enabled {
            if let Some(room_index) = self.end_location.room_index {
                if let Some(door_id) = self.end_location.door_id {
                    if let Some(room) = rooms.get(&room_index){                        
                        if let Some(c_rect) = room.get_control_rect(&door_id, true) {
                            self.end = c_rect;
                        }
                    }
                }
            }
        }
    }
}

impl Meshable for HallWay {
    fn mesh(&self) -> Vec<Mesh> {
        let mut meshs = vec![];
        let mut start_c_rect = &self.start;
        for i in 0..=self.middle.len() {
            let end_c_rect = self.middle.get(i).map(|t| &t.0).unwrap_or(&(self.end));
            let start_texture = self
                .middle
                .get(i - 1)
                .map(|t| &t.1)
                .unwrap_or(&(self.start_texture));

            let mut hallway_mesh = start_c_rect.gen_mesh(end_c_rect, start_texture);
            meshs.append(&mut hallway_mesh);
            start_c_rect = end_c_rect
        }

        meshs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallWayTexData {
    pub top: MeshTex,
    pub bottom: MeshTex,
    pub left: MeshTex,
    pub right: MeshTex,
}

impl HallWayTexData {
    pub fn all(mesh_tex: MeshTex) -> Self {
        HallWayTexData {
            top: mesh_tex.clone(),
            bottom: mesh_tex.clone(),
            left: mesh_tex.clone(),
            right: mesh_tex,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorLocation {
    pub room_index: Option<RoomId>,
    pub door_id: Option<DoorId>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRect {
    pub position: Vector3<f32>,
    pub rotation: Deg<f32>,
    pub size: Vector2<f32>,
}

impl ControlRect {
    pub fn new(position: Vector3<f32>, rotation: Deg<f32>, size: Vector2<f32>) -> Self {
        Self {
            position,
            rotation,
            size,
        }
    }
    pub fn gen_mesh(&self, other: &Self, tex: &HallWayTexData) -> Vec<Mesh> {
        let mut meshs = vec![];
        let floor_points = vec![
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / 2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / -2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / 2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / -2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
        ];
        let floor_tex_coords = tex.bottom.get_tex_coords(
            &floor_points
                .iter()
                .map(|a| Into::<(_, _)>::into(a.xz()))
                .collect_vec(),
        );
        let floor_mesh = Mesh {
            textrure: tex.bottom.id.id.clone(),
            vertices: floor_points
                .iter()
                .enumerate()
                .map(|(i, p3)| MeshVertex {
                    position: (*p3).into(),
                    tex_coords: floor_tex_coords[i],
                })
                .collect_vec(),
            indices: vec![2, 1, 0, 2, 0, 3],
        };
        let roof_points = vec![
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / 2.))
                    .extend(self.size.y);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / -2.))
                    .extend(self.size.y);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / 2.))
                    .extend(other.size.y);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / -2.))
                    .extend(other.size.y);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
        ];
        let roof_tex_coords = tex.top.get_tex_coords(
            &roof_points
                .iter()
                .map(|a| Into::<(_, _)>::into(a.xz()))
                .collect_vec(),
        );
        let roof_mesh = Mesh {
            textrure: tex.top.id.id.clone(),
            vertices: roof_points
                .iter()
                .enumerate()
                .map(|(i, p3)| MeshVertex {
                    position: (*p3).into(),
                    tex_coords: roof_tex_coords[i],
                })
                .collect_vec(),
            indices: vec![0, 1, 2, 3, 0, 2],
        };
        let left_distance = ((Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
            * (self.size.x / -2.))
            + self.position.xz())
        .distance(
            (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                * (other.size.x / 2.))
                + other.position.xz(),
        );
        let left_tex_points = vec![
            Vector2::new(0., self.position.y),
            Vector2::new(0., self.position.y + self.size.y),
            Vector2::new(left_distance, other.position.y + other.size.y),
            Vector2::new(left_distance, other.position.y),
        ];
        let left_points = vec![
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / -2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / -2.))
                    .extend(self.size.y);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / 2.))
                    .extend(other.size.y);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / 2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
        ];
        let left_tex_coords = tex.left.get_tex_coords(
            &left_tex_points
                .iter()
                .map(|a| Into::<(_, _)>::into(*a))
                .collect_vec(),
        );
        let left_mesh = Mesh {
            textrure: tex.left.id.id.clone(),
            vertices: left_points
                .iter()
                .enumerate()
                .map(|(i, a)| MeshVertex {
                    position: (*a).into(),
                    tex_coords: left_tex_coords[i],
                })
                .collect_vec(),
            indices: [0, 2, 1, 0, 3, 2].to_vec(),
        };
        let right_distance = ((Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
            * (self.size.x / 2.))
            + self.position.xz())
        .distance(
            (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                * (other.size.x / -2.))
                + other.position.xz(),
        );
        let right_tex_points = vec![
            Vector2::new(0., self.position.y),
            Vector2::new(0., self.position.y + self.size.y),
            Vector2::new(right_distance, other.position.y + other.size.y),
            Vector2::new(right_distance, other.position.y),
        ];
        let right_points = vec![
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / 2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(Deg(180.)-self.rotation).rotate_vector(Vector2::unit_x())
                    * (self.size.x / 2.))
                    .extend(self.size.y);
                p.swap_elements(1, 2);
                p += self.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / -2.))
                    .extend(other.size.y);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
            {
                let mut p = (Basis2::from_angle(-other.rotation).rotate_vector(Vector2::unit_x())
                    * (other.size.x / -2.))
                    .extend(0.);
                p.swap_elements(1, 2);
                p += other.position;
                p
            },
        ];
        let right_tex_coords = tex.right.get_tex_coords(
            &right_tex_points
                .iter()
                .map(|a| Into::<(_, _)>::into(*a))
                .collect_vec(),
        );
        let right_mesh = Mesh {
            textrure: tex.right.id.id.clone(),
            vertices: right_points
                .iter()
                .enumerate()
                .map(|(i, a)| MeshVertex {
                    position: (*a).into(),
                    tex_coords: right_tex_coords[i],
                })
                .collect_vec(),
            indices: [2, 3, 0, 2, 0, 1].to_vec(),
        };
        meshs.push(floor_mesh);
        meshs.push(roof_mesh);
        meshs.push(left_mesh);
        meshs.push(right_mesh);
        meshs
    }
}
