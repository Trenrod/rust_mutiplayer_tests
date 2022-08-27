mod gameclient;

use gameclient::{Game, ServerConnection};
use libs::messages::GameServerMessage;
use macroquad::window::next_frame;
use url::Url;

mod game {}

// Websocket client test
#[macroquad::main("game")]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    log::debug!("Client initialising...");

    // Create dedicated thread for server-client communication
    let game_server_url = Url::parse("ws://127.0.0.1:3030/game").unwrap();
    let mut server_connection = ServerConnection::new();
    server_connection.connect(game_server_url);

    let mut game = Game::new().await;

    loop {
        if let Some(msg) = server_connection.poll() {
            let message = match serde_json::from_slice::<GameServerMessage>(&msg) {
                Ok(message) => Some(message),
                _ => None,
            };

            if let Some(gs_message) = message {
                game.update_from_server(gs_message);
            }
        }

        server_connection.push(&game.local_player_state);

        game.update();

        game.draw();

        if game.quit {
            return;
        }

        next_frame().await;
    }
}
