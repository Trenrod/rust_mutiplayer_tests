use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct RemoteState {
    pub id: u16,
    pub position: Vec2,
    pub rotation: f32,
}
