use std::collections::HashMap;

use crate::{camer_control::CameraController, renderer::camera::Camera};

use super::{
    blocks::{
        blocks::Blocks,
        full_block::FullBlock,
        meshable::{Mesh, Meshable},
    },
    spatial_grid::{Position, SpatialGrid},
};
use cgmath::{Basis3, Point3, Rad, Rotation3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    pub grid: SpatialGrid,
    pub start_camera: CameraController,
}

impl LevelData {
    pub fn new() -> Self {
        let mut grid: HashMap<Position, (Basis3<f32>, Blocks)> = HashMap::new();
        fn place_block(hashmap: &mut HashMap<Position, (Basis3<f32>, Blocks)>, pos:Position){
            hashmap.insert(
                pos,
                (
                    Basis3::from_angle_x(Rad(0.)),
                    Blocks::FullBlock(FullBlock::new("default".into())),
                ),
            );
        }
        //floor
        place_block(&mut grid, Position::new(0, 0, 0));
        place_block(&mut grid, Position::new(-1, 0, 0));
        place_block(&mut grid, Position::new(-1, 0, -1));
        place_block(&mut grid, Position::new(0, 0, -1));
        //roof
        place_block(&mut grid, Position::new(0, 3, 0));
        place_block(&mut grid, Position::new(-1, 3, 0));
        place_block(&mut grid, Position::new(-1, 3, -1));
        place_block(&mut grid, Position::new(0, 3, -1));
        

        Self {
            grid,
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
    pub grid: SpatialGrid,
    pub cached_meshs: HashMap<Position, Vec<Mesh>>,
}

impl LevelState {
    pub fn from_level_data(data: &LevelData) -> Self {
        Self {
            camera_controler: data.start_camera.clone(),
            grid: data.grid.clone(),
            cached_meshs: HashMap::new(),
        }
    }
    pub fn mesh_single(&mut self, pos: &Position) {
        if let Some((rot, meshable)) = self.grid.get(&pos) {
            self.cached_meshs.insert(pos.clone(), {
                let mut meshs = meshable.mesh();
                meshs.iter_mut().for_each(|mesh| {
                    mesh.rotate(rot);
                    mesh.move_to(pos, 1.0);
                });
                meshs
            });
        }
    }
    pub fn mesh_all(&mut self) {
        self.grid.iter().for_each(|(pos, (rot, meshable))| {
            self.cached_meshs.insert(pos.clone(), {
                let mut meshs = meshable.mesh();
                meshs.iter_mut().for_each(|mesh| {
                    mesh.rotate(rot);
                    mesh.move_to(pos, 1.0);
                });
                meshs
            });
        });
    }
}
