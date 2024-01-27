use cgmath::{Array, Matrix2, Rad, Vector2, Vector3};
use earcutr;
use serde::{Deserialize, Serialize};

use super::mesh::{Mesh, MeshTex, MeshVertex, Meshable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub position: Vector3<f32>,
    pub rotation: Rad<f32>,
    pub walls: Vec<Wall>,
    pub doors: Vec<Door>,
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
        floor_texture: &MeshTex,
        roof_texture: &MeshTex,
        wall_texture: &MeshTex,
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
            doors: vec![],
            floor_texture: floor_texture.clone(),
            roof_texture: roof_texture.clone(),
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
            acc + ((points[1][0] - points[0][0]) * (points[1][1] - points[0][1]))
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
        let roof_mesh_vertices = points.into_iter().fold(vec![], |mut acc, point| {
            let mut position = (Matrix2::from_angle(self.rotation)
            * Vector2 {
                x: point[0],
                y: point[1],
            })
            .extend(0.0);
            position.swap_elements(1, 2);
            position += self.position;
            //position.swap_elements(1, 2);
            acc.push(MeshVertex {
                position: Into::<[f32; 3]>::into(position),
                tex_coords: self.floor_texture.clone().get_tex_coords(
                    top_right[0] - bottom_left[0],
                    top_right[1] - bottom_left[1],
                    point[0] - bottom_left[0],
                    point[1] - bottom_left[1],
                ),
            });
            acc
        });
        let floor_mesh_vertices = roof_mesh_vertices
            .iter()
            .map(|mesh_vertes| {
                let mut new = mesh_vertes.clone();
                new.position[1] += self.height;
                new
            })
            .collect::<Vec<MeshVertex>>();
        let floor = Mesh {
            textrure: self.floor_texture.id.clone(),
            vertices: floor_mesh_vertices,
            indices: floor_indices,
        };
        let roof = Mesh {
            textrure: self.roof_texture.id.clone(),
            vertices: roof_mesh_vertices,
            indices: roof_indecies,
        };
        meshs.append(&mut vec![roof, floor]);

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
    pub offset: Vector2<f32>,
    pub size: Vector2<f32>,
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
