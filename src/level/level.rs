use std::collections::HashMap;

use crate::{camer_control::CameraController, renderer::camera::Camera};

use cgmath::{Basis3, Point3, Rad, Rotation3};
use serde::{Deserialize, Serialize};

use super::{mesh::{Meshable, Mesh}, control_rect::{ControlRectId, ControlRect}, hallway::HallWay};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub start_camera: CameraController,
    pub control_rects:HashMap<ControlRectId,ControlRect>,
    pub hallways:Vec<HallWay>,
}

impl LevelData {
    pub fn new() -> Self {
        let control_rects = HashMap::new();
        Self {
            start_camera: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Rad(0.0), Rad(0.0)),
            ),
            control_rects,
            hallways:vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelState {
    pub camera_controler: CameraController,
}

impl LevelState {
    pub fn from_level_data(data: &LevelData) -> Self {
        Self {
            camera_controler: data.start_camera.clone(),
        }
    }
}

impl Meshable for LevelState{
    fn mesh(&self)->Vec<Mesh>{
        vec![]
    }
}
