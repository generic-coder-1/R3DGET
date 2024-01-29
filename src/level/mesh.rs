use crate::renderer::{texture::TextureId, vertex::Vertex};
use serde::{Deserialize, Serialize};

pub trait Meshable {
    fn mesh(&self) -> Vec<Mesh>;
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub textrure: TextureId,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Mesh {
    pub fn vertices(&self) -> Vec<Vertex> {
        self.vertices
            .iter()
            .map(|mesh_vertex| Vertex {
                position: mesh_vertex.position,
                tex_coords: mesh_vertex.tex_coords,
            })
            .collect()
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTex {
    pub id: TextureId,
    pub offset: [f32; 2],
    pub tile: [f32; 2],
    pub fliped: [bool; 2],
}

impl MeshTex {
    pub fn new(id: TextureId) -> Self {
        Self {
            id,
            tile: [1., 1.],
            offset: [0., 0.],
            fliped: [false, false],
        }
    }
    pub fn get_tex_coords(&self, width: f32, height: f32, x: f32, y: f32) -> [f32; 2] {
        [
            (if self.fliped[0] { x } else { width-x } / width) * self.tile[0] + self.offset[0],
            (if self.fliped[1] { y } else { height-y } / height) * self.tile[1] + self.offset[0],
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TexFill {
    Stretch,
    TileSpecific([f32; 2]),
}
