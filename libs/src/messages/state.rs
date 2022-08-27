use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct State {
    pub id: u16,
    pub position: Vec2,
    pub rotation: f32,
}
