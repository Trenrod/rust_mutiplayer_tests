pub mod player;
pub use crate::gameclient::player::LocalPlayer;
pub use crate::gameclient::player::Player;
pub use crate::gameclient::player::RemotePlayer;

pub mod server_connection;
pub use crate::gameclient::server_connection::ServerConnection;

pub mod game;
pub use crate::gameclient::game::Game;
