use std::{collections::HashMap, f32::consts::PI, marker::PhantomData, ops::Deref, vec};

use cgmath::{
    num_traits::Signed, Array, Basis2, InnerSpace, Matrix2, MetricSpace, Deg, Rotation, Rotation2,
    Vector2, Vector3, VectorSpace,
};
use earcutr::{self, earcut};
use geo::{coord, BooleanOps, CoordsIter, MultiPolygon, Polygon, Rect};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uid::IdU16;

use crate::ModuloSignedExt;

use super::{
    hallway::ControlRect,
    mesh::{Mesh, MeshTex, MeshVertex, Meshable},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct RoomId(pub uid::IdU16<PhantomData<Room>>);

impl RoomId {
    pub fn new() -> Self {
        Self(IdU16::<PhantomData<Room>>::new())
    }
}

impl Default for RoomId{
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.get().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(
            //we get these values from the saved files so they should be safe(hopefully)
            unsafe { IdU16::<PhantomData<Room>>::new_unchecked(u16::deserialize(deserializer)?) },
        ))
    }
}

impl Deref for RoomId {
    type Target = IdU16<PhantomData<Room>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub position: Vector3<f32>,
    pub rotation: Deg<f32>,
    pub walls: Vec<Wall>,
    pub doors: HashMap<DoorId, Door>,
    pub height: f32,
    pub moddifiers: Vec<Modifier>,
    pub floor_texture: MeshTex,
    pub roof_texture: MeshTex,
    pub name: String,
}

impl Room {
    pub fn new(
        name: String,
        position: Vector3<f32>,
        rotation: Deg<f32>,
        height: f32,
        floor_texture: MeshTex,
        roof_texture: MeshTex,
        wall_texture: MeshTex,
    ) -> Self {
        Self {
            name,
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
    pub fn get_control_rect(&self, id: &DoorId, away_from: bool) -> Option<ControlRect> {
        let door = self.doors.get(id)?;
        let (start, end) = self
            .walls
            .iter()
            .circular_tuple_windows::<(&Wall, &Wall)>()
            .nth((door.wall.modulo(self.walls.len() as isize)) as usize)
            .expect("Wall doesn't exist");
        let (x, z) =
            (Matrix2::from_angle(self.rotation) * ({
                let dist = start.local_pos.distance(end.local_pos);
                match door.center.1{
                    HorizontalAlign::Center=>{(start.local_pos+end.local_pos)/2.}
                    HorizontalAlign::Left => {start.local_pos.lerp(end.local_pos, door.size.x/(2.*dist))},
                    HorizontalAlign::Right => {start.local_pos.lerp(end.local_pos, 1.-(door.size.x/2.)/(dist))},
                }
            })).into();
        let y = match door.center.0 {
            VerticalAlign::Top => self.height-door.size.y,
            VerticalAlign::Center => (self.height-door.size.y)/2.,
            VerticalAlign::Bottom => {0.},
        };

        let position = Vector3::new(x, y, z) + self.position;

        let mut rotation:Deg<f32> = Vector2::unit_x().angle((start.local_pos + end.local_pos)/2.).into();        
        rotation+=Deg(90.);
        if !away_from{
            rotation+=Deg(180.);
        }
        rotation-=self.rotation;
        Some(ControlRect {
            position,
            rotation: rotation,
            size: door.size,
        })
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

        let roof_indecies = earcutr::earcut(&e_points, &[], dim)
            .expect("floor didn't earcut properly :(")
            .into_iter()
            .map(|usize| usize as u16)
            .collect::<Vec<u16>>();
        let mut floor_indices = roof_indecies.clone();
        floor_indices.reverse();
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
        let roof_mesh_vertices: Vec<MeshVertex> = points3
        .iter()
        .enumerate()
        .fold(vec![], |mut acc, (i, point)| {
            acc.push(MeshVertex {
                position: Into::<[f32; 3]>::into(*point + Vector3::new(0., self.height, 0.)),
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
        let floor_mesh_vertices = points3
            .iter()
            .enumerate()
            .fold(vec![], |mut acc, (i, point)| {
                acc.push(MeshVertex {
                    position: Into::<[f32; 3]>::into(*point),
                    tex_coords: floor_tex_coords[i],
                });
                acc
            });
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
                acc[door.wall.modulo(self.walls.len() as isize) as usize].push(door);
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

                let wall_tex_coords = wall_1.wall_texture.get_tex_coords(
                    &e_points
                        .iter()
                        .tuples::<(_, _)>()
                        .map(|a| (*a.0, *a.1))
                        .collect_vec(),
                ); //points3.iter().map(|point|{Into::<[f32;2]>::into(point.xy())}).collect_vec();

                let wall_mesh = points3.into_iter().enumerate().fold(
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
        self.moddifiers.iter().for_each(|modifer| {
            meshs.append(&mut (modifer.gen_mesh(self.position, self.rotation, self.height)));
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
        dir: Deg<f32>,
        size: Vector3<f32>,
        ramp_texture: MeshTex,
        wall_texture: MeshTex,
        bottom_texture: MeshTex,
    },
    Cliff {
        walls: Vec<Wall>,
        on_roof: bool,
        height: f32,
        floor_texture: MeshTex,
    },
    Disc {
        pos: Vector3<f32>,
        size: Vector3<f32>,
        sides: Vec<MeshTex>,
        dir: Deg<f32>,
        top_tex: MeshTex,
        bottom_tex: MeshTex,
    },
}

impl PartialEq for Modifier{
    fn eq(&self, other: &Self) -> bool {
        match (self,other){
            (Modifier::Ramp {..}, Modifier::Ramp {..}) => true,
            (Modifier::Cliff {..}, Modifier::Cliff {..}) => true,
            (Modifier::Disc {..}, Modifier::Disc {..}) => true,
            _=>false
        }
    }
}

impl Modifier {
    pub fn gen_mesh(
        &self,
        true_position: Vector3<f32>,
        true_dir: Deg<f32>,
        room_height: f32,
    ) -> Vec<Mesh> {
        let mut meshs = vec![];
        match self {
            Modifier::Ramp {
                pos,
                dir,
                size,
                ramp_texture,
                wall_texture,
                bottom_texture,
            } => {
                let ramp_distance = size.zy().distance(Vector2::new(0., 0.));
                let ramp_points = vec![
                    Vector3::new(size.x, 0., 0.),
                    Vector3::new(size.x, size.z, size.y),
                    Vector3::new(0., size.z, size.y),
                    Vector3::new(0., 0., 0.),
                ];
                let ramp_tex_coords = ramp_texture.get_tex_coords(
                    &[
                        Vector2::new(size.x, 0.),
                        Vector2::new(size.x, ramp_distance),
                        Vector2::new(0., ramp_distance),
                        Vector2::new(0., 0.),
                    ]
                    .iter()
                    .map(|point2| Into::<[f32; 2]>::into(*point2).into())
                    .collect_vec(),
                );
                let ramp_mesh = Mesh {
                    textrure: ramp_texture.id.id.clone(),
                    vertices: ramp_points
                        .iter()
                        .enumerate()
                        .map(|(i, point2)| MeshVertex {
                            position: {
                                let mut position: Vector3<f32> = Basis2::from_angle(dir.clone())
                                    .rotate_vector(point2.xy())
                                    .extend(point2.z);
                                position.swap_elements(1, 2);
                                position += *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                Into::<[f32; 3]>::into(position)
                            },
                            tex_coords: ramp_tex_coords[i],
                        })
                        .collect_vec(),
                    indices: vec![2, 1, 0, 3, 2, 0],
                };
                meshs.push(ramp_mesh);
                let front_tex_points = wall_texture.get_tex_coords(&vec![
                    (0., 0.),
                    (size.x, 0.),
                    (size.x, size.y),
                    (0., size.y),
                ]);
                let front_mesh = Mesh {
                    textrure: wall_texture.id.id.clone(),
                    vertices: [(0., 0.), (size.x, 0.), (size.x, size.y), (0., size.y)]
                        .into_iter()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position: Vector3<f32> =
                                    Vector3::new(point.0, point.1, size.z);
                                position = Basis2::from_angle(*dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                Into::<[f32; 3]>::into(position)
                            },
                            tex_coords: front_tex_points[i],
                        })
                        .collect_vec(),
                    indices: vec![2, 3, 0, 1, 2, 0],
                };
                meshs.push(front_mesh);
                let left_tex_point =
                    wall_texture.get_tex_coords(&vec![(0., 0.), (size.z, 0.), (size.z, size.y)]);
                let left_mesh = Mesh {
                    textrure: wall_texture.id.id.clone(),
                    vertices: [(0., 0.), (size.z, 0.), (size.z, size.y)]
                        .into_iter()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position: Vector3<f32> = Basis2::from_angle(*dir)
                                    .rotate_vector(Vector2::new(size.x, point.0))
                                    .extend(point.1);
                                position.swap_elements(1, 2);
                                position += *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                Into::<[f32; 3]>::into(position)
                            },
                            tex_coords: left_tex_point[i],
                        })
                        .collect_vec(),
                    indices: vec![0, 2, 1],
                };
                meshs.push(left_mesh);
                let right_mesh = Mesh {
                    textrure: wall_texture.id.id.clone(),
                    vertices: [(0., 0.), (size.z, 0.), (size.z, size.y)]
                        .into_iter()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position: Vector3<f32> = Basis2::from_angle(*dir)
                                    .rotate_vector(Vector2::new(0., point.0))
                                    .extend(point.1);
                                position.swap_elements(1, 2);
                                position += *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                Into::<[f32; 3]>::into(position)
                            },
                            tex_coords: left_tex_point[i],
                        })
                        .collect_vec(),
                    indices: vec![1, 2, 0],
                };
                meshs.push(right_mesh);
                let bottom_tex_points = bottom_texture.get_tex_coords(&vec![
                    (0., 0.),
                    (size.x, 0.),
                    (size.x, size.z),
                    (0., size.z),
                ]);
                let bottom_mesh = Mesh {
                    textrure: bottom_texture.id.id.clone(),
                    vertices: [(0., 0.), (size.x, 0.), (size.x, size.z), (0., size.z)]
                        .iter_mut()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position = Basis2::from_angle(*dir)
                                    .rotate_vector(Vector2::new(point.0, point.1))
                                    .extend(0.);
                                position.swap_elements(1, 2);
                                position += *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                Into::<[f32; 3]>::into(position)
                            },
                            tex_coords: bottom_tex_points[i],
                        })
                        .collect_vec(),
                    indices: vec![2, 3, 0, 1, 2, 0],
                };
                meshs.push(bottom_mesh);
            }
            Modifier::Cliff {
                walls,
                on_roof,
                height,
                floor_texture,
            } => {
                if walls.len()>0{
                    let mut floor_points = walls.iter().map(|wall| wall.local_pos).collect_vec();
                    floor_points.iter_mut().for_each(|point| {
                        *point = Basis2::from_angle(true_dir).rotate_vector(*point);
                    });
                    let (e_points, e_holes, dims) = earcutr::flatten(&vec![floor_points
                        .iter()
                        .map(|point| vec![point.x, point.y])
                        .collect_vec()]);
                    let mut floor_indices = earcut(e_points.as_slice(), e_holes.as_slice(), dims)
                        .expect("modifer floor didn't earcut properly :(");
                    let floor_tex_points = floor_texture.get_tex_coords(
                        &floor_points
                            .iter()
                            .map(|point| Into::<[f32; 2]>::into(*point).into())
                            .collect_vec(),
                    );
                    if floor_points
                        .iter()
                        .circular_tuple_windows::<(_, _)>()
                        .fold(0., |acc, (point1, point2)| {
                            acc + ((point2.x - point1.x) * (point2.y + point1.y))
                        })
                        .is_negative()
                    {
                        floor_indices.reverse();
                    }
                    if *on_roof {
                        floor_indices.reverse();
                    }
                    let floor_mesh = Mesh {
                        textrure: floor_texture.id.id.clone(),
                        vertices: e_points
                            .into_iter()
                            .tuples()
                            .enumerate()
                            .map(|(i, (a, b))| MeshVertex {
                                position: {
                                    let mut position = Vector2::new(a, b).extend(if *on_roof {
                                        room_height - *height
                                    } else {
                                        *height
                                    });
                                    position.swap_elements(1, 2);
                                    position += true_position;
                                    Into::<[f32; 3]>::into(position)
                                },
                                tex_coords: floor_tex_points[i],
                            })
                            .collect_vec(),
                        indices: floor_indices
                            .into_iter()
                            .map(|index| index as u16)
                            .collect_vec(),
                    };
                    meshs.push(floor_mesh);
                    walls
                        .iter()
                        .circular_tuple_windows::<(_, _)>()
                        .for_each(|(wall_1, wall_2)| {
                            let top_right = [wall_1.local_pos.distance(wall_2.local_pos), *height];
                            let dir = (wall_2.local_pos - wall_1.local_pos).normalize();
                            let wall_points = vec![
                                [0., 0.],
                                [top_right[0], 0.],
                                Into::<[f32; 2]>::into(top_right),
                                [0., top_right[1]],
                            ];
                            let points3 = wall_points
                                .iter()
                                .map(|point2| {
                                    let y = point2[1]
                                        + true_position.y
                                        + if *on_roof { room_height - height } else { 0. };
                                    let (mut x, mut z) = (Matrix2::from_angle(true_dir)
                                        * (point2[0] * dir + wall_1.local_pos))
                                        .into();
                                    x += true_position.x;
                                    z += true_position.z;
                                    let position = Vector3::new(x, y, z);
                                    position
                                })
                                .collect_vec();

                            let wall_tex_coords = wall_1.wall_texture.get_tex_coords(
                                &wall_points.iter().map(|a| (a[0], a[1])).collect_vec(),
                            );

                            let wall_mesh = points3.into_iter().enumerate().fold(
                                Mesh {
                                    textrure: wall_1.wall_texture.id.id.clone(),
                                    vertices: vec![],
                                    indices: vec![0, 3, 2, 0, 2, 1],
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
                }
            }
            Modifier::Disc {
                pos,
                size,
                sides,
                dir,
                top_tex,
                bottom_tex,
            } => {
                let flat_points = sides
                    .iter()
                    .enumerate()
                    .map(|(side, _)| {
                        let a = (
                            ((side as f32 / sides.len() as f32) * 2. * PI + PI / 4.).cos() * size.x,
                            ((side as f32 / sides.len() as f32) * 2. * PI + PI / 4.).sin() * size.z,
                        );
                        Basis2::from_angle(*dir - Deg(PI / 4.))
                            .rotate_vector(Vector2::new(a.0, a.1))
                            .into()
                    })
                    .collect_vec();
                let mut flat_indecies = earcut(
                    flat_points
                        .iter()
                        .map(|a: &(f32, f32)| vec![a.0, a.1])
                        .flatten()
                        .collect_vec()
                        .as_slice(),
                    &[],
                    2,
                )
                .expect("disk didn't earcut properly :(")
                .into_iter()
                .map(|usize| usize as u16)
                .collect_vec();
                let top_tex_points = top_tex.get_tex_coords(&flat_points);
                let bottom_tex_points = bottom_tex.get_tex_coords(&flat_points);
                let bottom = Mesh {
                    textrure: bottom_tex.id.id.clone(),
                    vertices: flat_points
                        .iter()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position =
                                    Vector3::new(point.0, size.y / -2., point.1) + *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                position.into()
                            },
                            tex_coords: bottom_tex_points[i],
                        })
                        .collect_vec(),
                    indices: flat_indecies.clone(),
                };
                flat_indecies.reverse();
                let top = Mesh {
                    textrure: top_tex.id.id.clone(),
                    vertices: flat_points
                        .iter()
                        .enumerate()
                        .map(|(i, point)| MeshVertex {
                            position: {
                                let mut position =
                                    Vector3::new(point.0, size.y / 2., point.1) + *pos;
                                position = Basis2::from_angle(true_dir)
                                    .rotate_vector(position.xz())
                                    .extend(position.y);
                                position.swap_elements(1, 2);
                                position += true_position;
                                position.into()
                            },
                            tex_coords: top_tex_points[i],
                        })
                        .collect_vec(),
                    indices: flat_indecies.clone(),
                };
                meshs.append(&mut vec![top, bottom]);
                flat_points
                    .iter()
                    .circular_tuple_windows()
                    .enumerate()
                    .for_each(|(i, (point1, point2))| {
                        let side_distance = Vector2::new(point1.0, point1.1)
                            .distance(Vector2::new(point2.0, point2.1));
                        let side_tex_points = sides[i].get_tex_coords(&vec![
                            (0., pos.y - size.y / 2.),
                            (side_distance, pos.y - size.y / 2.),
                            (side_distance, pos.y + size.y / 2.),
                            (0., pos.y + size.y / 2.),
                        ]);
                        let side = Mesh {
                            textrure: sides[i].id.id.clone(),
                            vertices: [
                                Vector3::new(point1.0, -size.y / 2., point1.1),
                                Vector3::new(point2.0, -size.y / 2., point2.1),
                                Vector3::new(point2.0, size.y / 2., point2.1),
                                Vector3::new(point1.0, size.y / 2., point1.1),
                            ]
                            .into_iter()
                            .enumerate()
                            .map(|(i, point)| MeshVertex {
                                position: {
                                    let mut position = point + pos;
                                    position = Basis2::from_angle(true_dir)
                                        .rotate_vector(position.xz())
                                        .extend(position.y);
                                    position.swap_elements(1, 2);
                                    position += true_position;
                                    position.into()
                                },
                                tex_coords: side_tex_points[i],
                            })
                            .collect_vec(),
                            indices: vec![2, 1, 0, 3, 2, 0],
                        };
                        meshs.push(side);
                    });
            }
        };
        meshs
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Door {
    pub wall: isize,
    pub offset: Vector2<f32>,
    pub size: Vector2<f32>,
    pub center:(VerticalAlign,HorizontalAlign),
}

impl Door {
    pub fn to_rect(&self, width: f32, height: f32) -> Rect<f32> {
        let (top, bottom) = match self.center.0 {
            VerticalAlign::Top => (height + self.offset.y, height + self.offset.y - self.size.y),
            VerticalAlign::Center => (
                height / 2. + self.offset.y + self.size.y / 2.,
                height / 2. + self.offset.y - self.size.y / 2.,
            ),
            VerticalAlign::Bottom => (self.offset.y + self.size.y, self.offset.y),
        };
        let (left, right) = match self.center.1 {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct DoorId(pub uid::IdU16<PhantomData<Door>>);

impl DoorId {
    pub fn new() -> Self {
        Self(IdU16::<PhantomData<Door>>::new())
    }
}

impl Default for DoorId{
    fn default() -> Self {
        Self::new()
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

pub trait Chain {
    fn chain(self: Self, func: impl Fn(&mut Self)) -> Self;
}

impl<T> Chain for T {
    fn chain(mut self, func: impl Fn(&mut Self)) -> Self {
        func(&mut self);
        self
    }
}
