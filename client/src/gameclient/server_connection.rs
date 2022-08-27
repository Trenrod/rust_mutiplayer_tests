use std::net::TcpStream;

use libs::messages::{GameServerMessage, State};
use tungstenite::{connect, Message, WebSocket};

use super::LocalPlayer;

pub struct ServerConnection {
    socket: Option<WebSocket<TcpStream>>,
}

impl ServerConnection {
    pub fn new() -> Self {
        Self { socket: None }
    }

    pub fn connect(&mut self, url: url::Url) {
        if let Ok((mut socket, _)) = connect(url) {
            // if let Plain(s) = socket.get_mut() {
            // Make sure its an unblocking stream
            // To not block our game update loop
            let s = socket.get_mut();
            s.set_nonblocking(true).unwrap();
            // }
            self.socket = Some(socket);
        }
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        if let Some(socket) = &mut self.socket {
            if let Ok(msg) = socket.read_message() {
                if let Message::Binary(bin_data) = msg {
                    return Some(bin_data);
                }
            }
        }
        None
    }

    pub fn push(&mut self, local_player: &LocalPlayer) {
        if let Some(socket) = &mut self.socket {
            if socket.can_write() {
                let game_server_client_state = State {
                    id: local_player.player.id,
                    position: local_player.player.position,
                    rotation: local_player.player.rotation,
                };
                let new_state = GameServerMessage::ClientUpdate(game_server_client_state);
                let client_update_message = serde_json::to_vec(&new_state);
                match client_update_message {
                    Ok(bin_client_message) => {
                        if let Err(err) =
                            socket.write_message(tungstenite::Message::Binary(bin_client_message))
                        {
                            log::debug!("Failed to update client state. {}", err);
                        }
                    }
                    Err(error) => log::error!("Failed to parse client state {:?}", error),
                }
            }
        }
    }
}
