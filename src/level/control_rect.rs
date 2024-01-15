use serde::{Serialize, Deserialize};

pub type ControlRectId = Box<str>;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ControlRect {
    pub position: [f32; 3],
    pub rotation: f32,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl ControlRect {
    pub fn new(position: [f32; 3], rotation: f32, width: f32, height: f32) -> Self {
        Self {
            position,
            rotation,
            top: height/2.,
            bottom: height/2.,
            left: width/2.,
            right: width/2.,
        }
    }
}
