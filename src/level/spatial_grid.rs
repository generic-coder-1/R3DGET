use std::collections::HashMap;

use cgmath::Basis3;
use serde::{Serialize, Deserialize};

use super::blocks::blocks::Blocks;
#[derive(PartialEq, Eq, Hash, Clone,Serialize,Deserialize, Debug)]
pub struct Position {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl Position {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self { x, y, z }
    }
}

impl Into<[f32; 3]> for &Position {
    fn into(self) -> [f32; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }
}


pub type SpatialGrid = HashMap<Position, (Basis3<f32>, Blocks)>;
