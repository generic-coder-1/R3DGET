use super::meshable::Meshable;

use serde::{Serialize, Deserialize};
#[derive(Serialize,Deserialize, Clone, Debug)]
pub struct AirBlock{}

impl Meshable for AirBlock{
    fn mesh(&self)->Vec<super::meshable::Mesh> {
        vec![]
    }
}