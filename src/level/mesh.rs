use serde::{Deserialize, Serialize};
use crate::renderer::{texture::TextureId, vertex::Vertex};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTex{
    pub id:TextureId,
    pub fill:TexFill,
    pub offset:[f32;2],
    pub fliped:bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TexFill{
    Stretch,
    TileSpecific([f32;2]),
    TileScale(f32),
}