use crate::level::mesh::MeshTex;

use super::control_rect::ControlRectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallWay {
    pub start: ControlRectId,
    pub others: Vec<HallWaySegment>,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallWaySegment {
    pub rect_id: ControlRectId,
    pub project: HallWayTexProject
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HallWayTexProject{
    Flat(MeshTex),
    Regular{
        sides: MeshTex,
        roof: MeshTex,
        floor: MeshTex,
    },
}

impl HallWay {
    pub fn new(start: ControlRectId, end: ControlRectId, roof: MeshTex, floor:MeshTex, sides:MeshTex) -> Self {
        Self {
            start,
            others: vec![HallWaySegment {
                rect_id: end,
                project:HallWayTexProject::Regular { sides, roof, floor }
            }],
        }
    }
}
