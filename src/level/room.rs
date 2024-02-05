use std::{collections::HashMap, marker::PhantomData, ops::Deref, vec};

use cgmath::{Array, InnerSpace, Matrix2, MetricSpace, Rad, Vector2, Vector3};
use earcutr;
use geo::{coord, BooleanOps, CoordsIter, MultiPolygon, Polygon, Rect};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uid::IdU16;

use super::{
    hallway::ControlRect,
    mesh::{Mesh, MeshTex, MeshVertex, Meshable},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub position: Vector3<f32>,
    pub rotation: Rad<f32>,
    pub walls: Vec<Wall>,
    pub doors: HashMap<DoorId, Door>,
    pub height: f32,
    pub moddifiers: Vec<Modifier>,
    pub floor_texture: MeshTex,
    pub roof_texture: MeshTex,
}

impl Room {
    pub fn new(
        position: Vector3<f32>,
        rotation: Rad<f32>,
        height: f32,
        floor_texture: MeshTex,
        roof_texture: MeshTex,
        wall_texture: MeshTex,
    ) -> Self {
        Self {
            position,
            height,
            rotation,
            walls: vec![
                Wall::new([-1., -1.].into(), wall_texture.clone()),
                Wall::new([1., -1.].into(), wall_texture.clone()),
                Wall::new([1., 1.].into(), wall_texture.clone()),
                Wall::new([-1., 1.].into(), wall_texture.clone()),
            ],
            moddifiers: vec![],
            doors: HashMap::new(),
            floor_texture: floor_texture.clone(),
            roof_texture: roof_texture.clone(),
        }
    }
    pub fn new_door(&mut self, door: Door) -> DoorId {
        let id = DoorId::new();
        self.doors.insert(id.clone(), door);
        id
    }
    pub fn get_control_rect(&self, id: &DoorId, away_from: bool) -> ControlRect {
        let door = self.doors.get(id).expect("door doesn't exist");
        let (start, end) = self
            .walls
            .iter()
            .circular_tuple_windows::<(&Wall, &Wall)>()
            .nth(door.wall)
            .expect("Wall doesn't exist");
        let (x, z) =
            (Matrix2::from_angle(self.rotation) * ((start.local_pos + end.local_pos) / 2.)).into();
        let y = 0.;

        let position = Vector3::new(x, y, z) + self.position;

        let wall_midpoint = Matrix2::from_angle(self.rotation) * (end.local_pos - start.local_pos);
        let rotation = if away_from {
            Vector2::unit_x().angle(wall_midpoint)
        } else {
            wall_midpoint.angle(Vector2::unit_x())
        }
        .0;
        ControlRect {
            position,
            rotation:Rad(rotation),
            size: door.size,
        }
    }
}

impl Meshable for Room {
    fn mesh(&self) -> Vec<super::mesh::Mesh> {
        let mut meshs: Vec<Mesh> = vec![];

        //floor and roof
        let mut bottom_left = [f32::INFINITY, f32::INFINITY];
        let mut top_right = [f32::NEG_INFINITY, f32::NEG_INFINITY];
        let mut points = self
            .walls
            .iter()
            .map(|wall| {
                let point = Into::<[f32; 2]>::into(wall.local_pos);
                if point[0] < bottom_left[0] {
                    bottom_left[0] = point[0]
                }
                if point[1] < bottom_left[1] {
                    bottom_left[1] = point[1]
                }
                if point[0] > top_right[0] {
                    top_right[0] = point[0]
                }
                if point[1] > top_right[1] {
                    top_right[1] = point[1]
                }

                point.into()
            })
            .collect::<Vec<Vec<f32>>>();
        points.push(points.first().unwrap().clone());
        let signed_area = points.windows(2).fold(0.0, |acc, points| {
            acc + ((points[1][0] - points[0][0]) * (points[1][1] + points[0][1]))
        });
        points.pop();
        if signed_area.is_sign_negative() {
            points.reverse();
        }

        let mut input_data = vec![points, vec![]];
        let (e_points, _, dim) = earcutr::flatten(&input_data);

        points = input_data.remove(0);

        let floor_indices = earcutr::earcut(&e_points, &[], dim)
            .expect("floor didn't earcut properly :(")
            .into_iter()
            .map(|usize| usize as u16)
            .collect::<Vec<u16>>();
        let mut roof_indecies = floor_indices.clone();
        roof_indecies.reverse();
        let points3 = points
            .iter()
            .map(|point2| {
                let mut posititon = (Matrix2::from_angle(self.rotation)
                    * Vector2 {
                        x: point2[0],
                        y: point2[1],
                    })
                .extend(0.0);
                posititon.swap_elements(1, 2);
                posititon += self.position;
                posititon
            })
            .collect_vec();
        let roof_tex_coords = self.roof_texture.get_tex_coords(
            &points3
                .iter()
                .map(|point| Into::<(f32, f32)>::into(point.xz()))
                .collect_vec(),
        );
        let roof_mesh_vertices = points3
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (i, point)| {
                acc.push(MeshVertex {
                    position: Into::<[f32; 3]>::into(*point),
                    tex_coords: roof_tex_coords[i],
                });
                acc
            });
        let floor_tex_coords = self.floor_texture.get_tex_coords(
            &points3
                .iter()
                .map(|point| Into::<(f32, f32)>::into(point.xz()))
                .collect_vec(),
        );
        let floor_mesh_vertices = roof_mesh_vertices
            .iter()
            .enumerate()
            .map(|(i, mesh_vertes)| {
                let mut new = mesh_vertes.clone();
                new.position[1] += self.height;
                new.tex_coords = floor_tex_coords[i];
                new
            })
            .collect::<Vec<MeshVertex>>();
        let floor = Mesh {
            textrure: self.floor_texture.id.id.clone(),
            vertices: floor_mesh_vertices,
            indices: floor_indices,
        };
        let roof = Mesh {
            textrure: self.roof_texture.id.id.clone(),
            vertices: roof_mesh_vertices,
            indices: roof_indecies,
        };
        meshs.append(&mut vec![roof, floor]);

        //walls
        let vec_doors: Vec<Vec<&Door>> = self.doors.values().into_iter().fold(
            (0..self.walls.len())
                .into_iter()
                .map(|_| vec![])
                .collect_vec(),
            |mut acc, door| {
                acc[door.wall].push(door);
                acc
            },
        );
        self.walls
            .iter()
            .circular_tuple_windows::<(_, _)>()
            .enumerate()
            .for_each(|(index, (wall_1, wall_2))| {
                let top_right = [wall_1.local_pos.distance(wall_2.local_pos), self.height];
                let dir = (wall_2.local_pos - wall_1.local_pos).normalize();
                let doors = &vec_doors[index];
                let wall_points = vec![
                    vec![0., 0.],
                    vec![top_right[0], 0.],
                    top_right.to_vec(),
                    vec![0., top_right[1]],
                ];
                let mut holes = doors
                    .iter()
                    .fold(MultiPolygon::new(vec![]), |acc, door: &&Door| {
                        let rect = door.to_rect(top_right[0], top_right[1]);
                        acc.union(&MultiPolygon::new(vec![Polygon::from(rect)]))
                    })
                    .into_iter()
                    .map(|polygon| {
                        polygon
                            .exterior_coords_iter()
                            .map(|coord| vec![coord.x, coord.y])
                            .collect_vec()
                    })
                    .collect_vec();
                let mut temp_input = vec![wall_points];
                temp_input.append(&mut holes);
                let (e_points, e_holes, dim) = earcutr::flatten(&temp_input);
                let wall_indecies = earcutr::earcut(e_points.as_slice(), &e_holes, dim)
                    .expect("wall didn't earcut properly :(");

                let points3 = e_points
                    .iter()
                    .tuples::<(_, _)>()
                    .map(|point2| {
                        let y = point2.1 + self.position.y;
                        let (mut x, mut z) = (Matrix2::from_angle(self.rotation)
                            * (*point2.0 * dir + wall_1.local_pos))
                            .into();
                        x += self.position.x;
                        z += self.position.z;
                        let position = Vector3::new(x, y, z);
                        position
                    })
                    .collect_vec();

                let wall_tex_coords = wall_1.wall_texture.get_tex_coords(&e_points.iter().tuples::<(_,_)>().map(|a|{(*a.0,*a.1)}).collect_vec());//points3.iter().map(|point|{Into::<[f32;2]>::into(point.xy())}).collect_vec();

                let wall_mesh = points3
                    .into_iter()
                    .enumerate()
                    .fold(
                        Mesh {
                            textrure: wall_1.wall_texture.id.id.clone(),
                            vertices: vec![],
                            indices: wall_indecies
                                .into_iter()
                                .map(|usize| usize as u16)
                                .collect_vec(),
                        },
                        |mut acc, (i, point)| {
                            acc.vertices.push(MeshVertex {
                                position: point.into(),
                                tex_coords: wall_tex_coords[i],
                            });
                            acc
                        },
                    );
                meshs.push(wall_mesh);
            });

        meshs
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wall {
    pub local_pos: Vector2<f32>,
    pub wall_texture: MeshTex,
}

impl Wall {
    pub fn new(pos: Vector2<f32>, wall_texture: MeshTex) -> Self {
        Self {
            local_pos: pos,
            wall_texture,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Modifier {
    Ramp {
        pos: Vector3<f32>,
        dir: f32,
        size: Vector3<f32>,
        floor_texture: MeshTex,
        wall_texture: MeshTex,
    },
    Cliff {
        walls: Vec<Wall>,
        height: f32,
        floor_texture: MeshTex,
        wall_texture: MeshTex,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Door {
    pub wall: usize,
    pub offset: Vector2<f32>,
    pub size: Vector2<f32>,
    pub vertical_alignment: VerticalAlign,
    pub horizontal_alignment: HorizontalAlign,
}

impl Door {
    pub fn to_rect(&self, width: f32, height: f32) -> Rect<f32> {
        let (top, bottom) = match self.vertical_alignment {
            VerticalAlign::Top => (height + self.offset.y, height + self.offset.y - self.size.y),
            VerticalAlign::Center => (
                height / 2. + self.offset.y + self.size.y / 2.,
                height / 2. + self.offset.y - self.size.y / 2.,
            ),
            VerticalAlign::Bottom => (self.offset.y + self.size.y, self.offset.y),
        };
        let (left, right) = match self.horizontal_alignment {
            HorizontalAlign::Left => (self.offset.x, self.offset.x + self.size.x),
            HorizontalAlign::Center => (
                width / 2. + self.offset.x - self.size.x / 2.,
                width / 2. + self.offset.x + self.size.x / 2.,
            ),
            HorizontalAlign::Right => (width + self.offset.x - self.size.x, width + self.offset.x),
        };
        Rect::new(coord! {x:right,y:top}, coord! {x:left,y:bottom})
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DoorId(uid::IdU16<PhantomData<Door>>);

impl DoorId {
    pub fn new() -> Self {
        Self(IdU16::<PhantomData<Door>>::new())
    }
}

impl Serialize for DoorId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.get().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DoorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(
            //we get these values from the saved files so they should be safe(hopefully)
            unsafe { IdU16::<PhantomData<Door>>::new_unchecked(u16::deserialize(deserializer)?) },
        ))
    }
}

impl Deref for DoorId {
    type Target = IdU16<PhantomData<Door>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait Wraper<T>
where
    Self: Sized,
{
    fn to_arr<const N: usize>(self: Self) -> [T; N];
}

impl<T> Wraper<T> for Vec<T> {
    fn to_arr<const N: usize>(self) -> [T; N] {
        self.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Expected a Vec of length {} but it was {}", N, v.len())
        })
    }
}

trait Chain {
    fn chain(self: Self, func: impl Fn(&mut Self)) -> Self;
}

impl<T> Chain for T {
    fn chain(mut self, func: impl Fn(&mut Self)) -> Self {
        func(&mut self);
        self
    }
}
