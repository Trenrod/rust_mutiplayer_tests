use glam::Vec2;

pub struct Player {
    pub id: u16,
    pub position: Vec2,
    pub rotation: f32,
}

impl Player {
    pub fn new(id: u16, position: Vec2, rotation: f32) -> Self {
        Player {
            id: id,
            position: position,
            rotation: rotation,
        }
    }
}

pub struct LocalPlayer {
    pub player: Player,
}

pub struct RemotePlayer {
    pub player: Player,
}

impl RemotePlayer {
    pub fn new(id: u16, position: Vec2, rotation: f32) -> Self {
        RemotePlayer {
            player: Player::new(id, position, rotation),
        }
    }
}
