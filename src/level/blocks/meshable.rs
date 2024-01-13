use cgmath::{Basis3, Rotation};
use serde::{Deserialize, Serialize};

use crate::{
    level::spatial_grid::Position,
    renderer::{texture::TextureId, vertex::Vertex},
};

pub trait Meshable {
    fn mesh(&self) -> Vec<Mesh>;
}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Mesh {
    pub textrure: TextureId,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u16>,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct MeshVertex{
    pub position:[f32; 3],
    pub tex_coords:[f32; 2],
}

impl Mesh {
    pub fn vertices(&self)->Vec<Vertex>{
        self.vertices.iter().map(|mesh_vertex|Vertex{position:mesh_vertex.position,tex_coords:mesh_vertex.tex_coords}).collect()
    }
    pub fn combine(&mut self, mut other: Self) {
        if self.textrure != self.textrure {
            panic!();
        }
        self.indices.append(
            other
                .indices
                .into_iter()
                .map(|index| index + self.vertices.len() as u16)
                .collect::<Vec<u16>>()
                .as_mut(),
        );
        self.vertices.append(&mut other.vertices);
    }
    pub fn rotate(&mut self, rotation: &Basis3<f32>) {
        self.vertices.iter_mut().for_each(|vertex| {
            vertex.position.iter_mut().for_each(|pos| *pos -= 0.5);
            vertex.position = rotation.rotate_point(vertex.position.into()).into();
            vertex.position.iter_mut().for_each(|pos| *pos += 0.5);
        })
    }
    pub fn move_to(&mut self, position: &Position, scale: f32) {
        self.vertices.iter_mut().for_each(|vertex| {
            vertex
                .position
                .iter_mut()
                .enumerate()
                .for_each(|(index, val)| {
                    *val += match index {
                        0 => position.x,
                        1 => position.y,
                        2 => position.z,
                        _ => 0,
                    } as f32;
                    *val *= scale;
                });
        });
    }
}
