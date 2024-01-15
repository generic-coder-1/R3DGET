use crate::renderer::texture::TextureId;

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
    pub sides: TextureId,
    pub roof: TextureId,
    pub floor: TextureId,
    pub flat_project: bool,
}

impl HallWay {
    pub fn new(start: ControlRectId, end: ControlRectId, texture_id: TextureId) -> Self {
        Self {
            start,
            others: vec![HallWaySegment {
                rect_id: end,
                flat_project: false,
                sides: texture_id.clone(),
                roof: texture_id.clone(),
                floor: texture_id.clone(),
            }],
        }
    }
}
