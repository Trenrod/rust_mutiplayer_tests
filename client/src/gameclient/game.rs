use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use glam::Vec2;
use libs::messages::GameServerMessage;
use macroquad::prelude::{GREEN, RED};
use macroquad::text::draw_text;
use macroquad::texture::{draw_texture_ex, load_texture};
use macroquad::{
    color_u8,
    prelude::{is_key_down, Color, KeyCode, WHITE},
    texture::{DrawTextureParams, Texture2D},
    window::{clear_background, screen_height, screen_width},
};

use super::{LocalPlayer, Player, RemotePlayer};

pub struct Game {
    pub quit: bool,
    pub local_player_state: LocalPlayer,
    pub remote_player_state: HashMap<u16, RemotePlayer>,
    pub texture: Texture2D,
}

impl Game {
    pub async fn new() -> Self {
        let texture = load_texture("assets/plane.png").await.unwrap();
        Self {
            quit: false,
            local_player_state: LocalPlayer {
                player: Player {
                    id: 0,
                    position: Vec2::new(100.0, 100.0),
                    rotation: 0.0,
                },
            },
            remote_player_state: HashMap::default(),
            texture: texture,
        }
    }

    pub fn update(&mut self) {
        if is_key_down(KeyCode::Escape) {
            self.quit = true;
        }

        const ROT_SPEED: f32 = 0.015;

        if is_key_down(KeyCode::Right) {
            self.local_player_state.player.rotation += ROT_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            self.local_player_state.player.rotation -= ROT_SPEED;
        }

        const SPEED: f32 = 0.6;

        if is_key_down(KeyCode::Up) {
            self.local_player_state.player.position +=
                vec2_from_angle(self.local_player_state.player.rotation) * SPEED;
        }
        if is_key_down(KeyCode::Down) {
            self.local_player_state.player.position -=
                vec2_from_angle(self.local_player_state.player.rotation) * SPEED;
        }

        if self.local_player_state.player.position.x > screen_width() {
            self.local_player_state.player.position.x = -self.texture.width();
        } else if self.local_player_state.player.position.x < -self.texture.width() {
            self.local_player_state.player.position.x = screen_width();
        }
        if self.local_player_state.player.position.y > screen_height() {
            self.local_player_state.player.position.y = -self.texture.height();
        } else if self.local_player_state.player.position.y < -self.texture.height() {
            self.local_player_state.player.position.y = screen_height();
        }
    }

    pub fn draw(&self) {
        clear_background(color_u8!(255, 255, 255, 255));

        self.draw_player(&self.local_player_state.player.id, &self.local_player_state);
        for (player_id, player_state) in &self.remote_player_state {
            self.draw_enemy(player_id, &player_state);
        }
    }

    fn draw_player(&self, player_id: &u16, local_player_state: &LocalPlayer) {
        draw_texture_ex(
            self.texture,
            local_player_state.player.position.x,
            local_player_state.player.position.y,
            WHITE,
            DrawTextureParams {
                rotation: local_player_state.player.rotation,
                ..Default::default()
            },
        );

        let text = format!("Player {}", player_id);
        draw_text(
            &text,
            local_player_state.player.position.x,
            local_player_state.player.position.y,
            16.0,
            GREEN,
        );
    }

    pub fn draw_enemy(&self, player_id: &u16, remote_player_state: &RemotePlayer) {
        draw_texture_ex(
            self.texture,
            remote_player_state.player.position.x,
            remote_player_state.player.position.y,
            WHITE,
            DrawTextureParams {
                rotation: remote_player_state.player.rotation,
                ..Default::default()
            },
        );

        let text = format!("Player {}", player_id);
        draw_text(
            &text,
            remote_player_state.player.position.x,
            remote_player_state.player.position.y,
            16.0,
            RED,
        )
    }

    pub fn update_from_server(&mut self, message: GameServerMessage) {
        match message {
            GameServerMessage::Welcome(user_id) => {
                self.local_player_state.player.id = user_id;
            }
            GameServerMessage::Goodbye(user_id) => {
                if let Some(_) = self.remote_player_state.remove(&user_id) {
                    log::info!("Player with player id {} removed.", user_id);
                } else {
                    log::info!("No player with player id {} exists.", user_id);
                }
            }
            GameServerMessage::ServerUpdate(remove_player_states) => {
                for state in remove_player_states {
                    // Update states of remove and local player
                    if state.id == self.local_player_state.player.id {
                        self.local_player_state.player.position = state.position;
                        self.local_player_state.player.rotation = state.rotation;
                    } else {
                        // Create or update existing
                        let remote_ref = self.remote_player_state.entry(state.id);
                        match remote_ref {
                            Occupied(remote) => {
                                let mut remote_mut = remote.into_mut();
                                remote_mut.player.position = state.position;
                                remote_mut.player.rotation = state.rotation;
                            }
                            Vacant(remote) => {
                                remote.insert(RemotePlayer::new(
                                    state.id,
                                    state.position,
                                    state.rotation,
                                ));
                            }
                        };
                    }
                }
            }
            _ => (),
        }
    }
}

fn vec2_from_angle(angle: f32) -> Vec2 {
    let angle = angle - std::f32::consts::FRAC_PI_2;
    Vec2::new(angle.cos(), angle.sin())
}
