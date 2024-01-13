
use super::{full_block::FullBlock, air::AirBlock, meshable::Meshable};

use serde::{Serialize, Deserialize};
#[derive(Serialize,Deserialize,Clone, Debug)]
pub enum Blocks{
    FullBlock(FullBlock),
    AirBlock(AirBlock),
}

impl Meshable for &Blocks{
    fn mesh(&self)->Vec<super::meshable::Mesh> {
        match self{
            Blocks::FullBlock(m) => m.mesh(),
            Blocks::AirBlock(m) => m.mesh(),
        }
    }
}
