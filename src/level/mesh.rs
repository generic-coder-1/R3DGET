use crate::renderer::{
    texture::{TextureData, TextureId},
    vertex::Vertex,
};
use geo::{coord, Polygon, BoundingRect};
use itertools::Itertools;
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
    pub id: TextureData,
    pub offset: [f32; 2],
    pub tile: TileStyle,
    pub fliped: [bool; 2],
}

impl MeshTex {
    pub fn new(id: TextureData, tile: TileStyle) -> Self {
        Self {
            id,
            tile,
            offset: [0., 0.],
            fliped: [false, false],
        }
    }
    pub fn get_tex_coords(&self, points: &Vec<(f32, f32)>) -> Vec<[f32; 2]> {
        let bounds = Polygon::new(
            geo::LineString(
                points
                    .iter()
                    .map(|point| coord! {x:point.0.clone(),y:point.1.clone()})
                    .collect_vec(),
            ),
            vec![],
        ).bounding_rect().unwrap();

        let helper_closure: Box<dyn Fn(&f32,&f32)->[f32;2]> = match self.tile {
            TileStyle::TileSpecific(x_tiles, y_tiles) => Box::new(move |x,y|{
                [
                    (if !self.fliped[0] {bounds.width()*x_tiles - (x-bounds.min().x+self.offset[0])} else {x-bounds.min().x+self.offset[0]}/(bounds.width()*x_tiles)),
                    (if !self.fliped[1] {bounds.height()*y_tiles - (y-bounds.min().y+self.offset[1])} else {y-bounds.min().y+self.offset[1]}/(bounds.height()*y_tiles)),
                ]
            }),
            TileStyle::TileScale(scale, global) => {
                match global {
                    true=> Box::new(move |x: &f32,y: &f32|{
                        [
                            (if !self.fliped[0]{-x}else{*x} + self.offset[0]) / (self.id.width * scale),
                            (if !self.fliped[1]{-y}else{*y} + self.offset[1]) / (self.id.height * scale)
                        ]
                    }),
                    false=>Box::new(move |x: &f32,y: &f32|{
                        [
                            (if !self.fliped[0]{-x}else{*x} + self.offset[0]) / (self.id.width * scale),
                            (if !self.fliped[1]{-y}else{*y} + self.offset[1]) / (self.id.height * scale)
                        ]
                    })
                }
            },
        };
        points.iter().map(|point|{
            helper_closure(&point.0,&point.1)
        }).collect_vec()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileStyle {
    TileSpecific(f32, f32),
    TileScale(f32, bool),
}
