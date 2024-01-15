use std::collections::HashMap;

use crate::{camer_control::CameraController, renderer::camera::Camera};

use cgmath::{Basis3, Point3, Rad, Rotation3};
use serde::{Deserialize, Serialize};

use super::mesh::{Meshable, Mesh};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub start_camera: CameraController,
}

impl LevelData {
    pub fn new() -> Self {
        Self {
            start_camera: CameraController::new(
                4.0,
                0.4,
                Camera::new(Point3::new(0.0, 2.0, 0.0), Rad(0.0), Rad(0.0)),
            ),
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
