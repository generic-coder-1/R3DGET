use crate::renderer::texture::TextureId;

use super::meshable::{Mesh, Meshable,MeshVertex};

use serde::{Serialize, Deserialize};
#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct FullBlock {
    pub top: TextureId,
    pub bottom: TextureId,
    pub left: TextureId,
    pub right: TextureId,
    pub front: TextureId,
    pub back: TextureId,
}

impl FullBlock {
    pub fn new(texture: TextureId) -> Self {
        Self {
            top: texture.clone(),
            bottom: texture.clone(),
            left: texture.clone(),
            right: texture.clone(),
            front: texture.clone(),
            back: texture.clone(),
        }
    }
}

impl Meshable for FullBlock {
    fn mesh(&self) -> Vec<Mesh> {
        let top: Mesh = Mesh {
            textrure: self.top.clone(),
            vertices: vec![
                MeshVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        let left: Mesh = Mesh {
            textrure: self.left.clone(),
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
                MeshVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        let bottom: Mesh = Mesh {
            textrure: self.bottom.clone(),
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
                MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        let right: Mesh = Mesh {
            textrure: self.right.clone(),
            vertices: vec![
                MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        let back: Mesh = Mesh {
            textrure: self.back.clone(),
            vertices: vec![
                MeshVertex {
                    position: [1.0, 0.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 1.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [0.0, 0.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                MeshVertex {
                    position: [0.0, 1.0, 1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        let front: Mesh = Mesh {
            textrure: self.front.clone(),
            vertices: vec![
                MeshVertex {
                    position: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                MeshVertex {
                    position: [0.0, 1.0, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                MeshVertex {
                    position: [1.0, 0.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
                MeshVertex {
                    position: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
            indices: vec![1, 2, 0, 1, 3, 2],
        };
        vec![top, front, left, bottom, right, back]
    }
}
