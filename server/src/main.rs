use std::collections::HashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use glam::Vec2;
use libs::messages::{GameServerMessage, RemoteState};
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;
use warp::ws::{Message, WebSocket};
use warp::Filter;

// Type alias allows to simplify readability
type OutBoundChannel = mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>;

// Our global unique user id counter
static NEXT_USER_ID: AtomicU16 = AtomicU16::new(1);

// Atomic reference to all channels of the currently connected clients
type MapUsers = Arc<RwLock<HashMap<u16, OutBoundChannel>>>;
type MapPlayerStates = Arc<RwLock<HashMap<u16, RemoteState>>>;

fn create_send_channel(
    ws_sender: futures_util::stream::SplitSink<WebSocket, Message>,
) -> OutBoundChannel {
    use futures_util::FutureExt;
    use futures_util::StreamExt;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    // Creates new channels
    let (sender, receiver) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(receiver);

    // Spawn new async task
    tokio::task::spawn(rx.forward(ws_sender).map(|result| {
        if let Err(e) = result {
            log::error!("websocket send error: {}", e);
        }
    }));
    sender
}

async fn send_welcome(out: &OutBoundChannel) -> u16 {
    // Create a new unique id for this connection
    let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    // Wrap id into welcome message
    let states = libs::messages::GameServerMessage::Welcome(id);

    // Sends message over outbound channel
    send_msg(out, &states).await;

    id
}

async fn send_msg(tx: &OutBoundChannel, msg: &GameServerMessage) {
    let buffer = serde_json::to_vec(msg).unwrap();
    // Wrap raw data into something websocket can send
    let ws_message = Message::binary(buffer);
    // Sending message via outbound channel
    tx.send(Ok(ws_message)).unwrap();
}

async fn user_connected(ws: WebSocket, users: MapUsers, states: MapPlayerStates) {
    use futures_util::StreamExt;

    // Split the socket into a sender and a receiver
    let (ws_sender, mut ws_receiver) = ws.split();
    // everything we send via this channel will be forwarded
    let send_channel = create_send_channel(ws_sender);
    // send welcome message by using the 'send_channel'
    let my_id = send_welcome(&send_channel).await;

    log::debug!("new user connected: {}", my_id);
    // Add user and the channel how to reach it to the global map
    users.write().await.insert(my_id, send_channel);
    states.write().await.insert(
        my_id,
        RemoteState {
            id: my_id,
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
        },
    );

    // Loop to get message over the clients receiver channel
    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                log::warn!("websocket received errror: '{}'", e);
                break;
            }
        };
        // Log messages from client
        if let Some(msg) = parse_message(msg) {
            user_message(my_id, msg, &states).await;
        }
    }
    // Log disconnection of client
    log::debug!("user disconnected: {}", my_id);
    // Remove our own entry from global map again so wie do not receive more messages
    users.write().await.remove(&my_id);
    states.write().await.remove(&my_id);
    // Send goodbye message to all remaining connected clients
    broadcast(GameServerMessage::Goodbye(my_id), &users).await;
}

async fn user_message(my_id: u16, msg: GameServerMessage, states: &MapPlayerStates) {
    match msg {
        GameServerMessage::ClientUpdate(state) => {
            let msg = RemoteState {
                id: my_id,
                position: state.position,
                rotation: state.rotation,
            };
            states.write().await.insert(msg.id, msg);
        }
        _ => (),
    }
}

fn parse_message(msg: Message) -> Option<GameServerMessage> {
    if msg.is_binary() {
        let msg = msg.into_bytes();
        serde_json::from_slice::<GameServerMessage>(msg.as_slice()).ok()
    } else {
        None
    }
}

async fn broadcast(msg: GameServerMessage, users: &MapUsers) {
    for (_, tx) in users.read().await.iter() {
        send_msg(tx, &msg).await;
    }
}

async fn update_loop(
    map_user_updateloop: Arc<RwLock<HashMap<u16, OutBoundChannel>>>,
    map_state_updateloop: Arc<RwLock<HashMap<u16, RemoteState>>>,
) {
    loop {
        // Ideal server->client refresh rate
        let max_duration = Duration::from_millis(50);
        let time_now = Instant::now();
        let states: Vec<RemoteState> = map_state_updateloop
            .read()
            .await
            .values()
            .cloned()
            .collect();
        if !states.is_empty() {
            for (_, tx) in map_user_updateloop.read().await.iter() {
                let update_states = GameServerMessage::ServerUpdate(states.clone());
                send_msg(tx, &update_states).await;
            }
        }

        let elapsed_sending_time = time_now.elapsed();
        let mut sleep_duration = Duration::from_millis(5);
        if elapsed_sending_time < max_duration {
            sleep_duration = max_duration - elapsed_sending_time;
        }
        // log::debug!("Sleep duration: {:?}", sleep_duration);
        tokio::time::sleep(sleep_duration).await;
    }
}

#[tokio::main] // Allows to annotate async
async fn main() {
    // Enables automatic loggin by warp
    pretty_env_logger::init();

    log::debug!("Server initialising...");

    // map of userid to sender channel
    let map_user_senderchannel = MapUsers::default();
    // map of userid to player state
    let map_user_playerstate = MapPlayerStates::default();

    // Create a thread which redulary upates all connected clients with all state
    let map_user_updateloop = map_user_senderchannel.clone();
    let map_state_updateloop = map_user_playerstate.clone();
    tokio::spawn(async move { update_loop(map_user_updateloop, map_state_updateloop).await });

    // Creates clones of this list for all warp connections
    let map_user_senderchannel = warp::any().map(move || map_user_senderchannel.clone());

    // Creates clones of this list for all warp connections
    let map_user_playerstate = warp::any().map(move || map_user_playerstate.clone());

    // Main game route for client-websocket connections
    let game = warp::path("game")
        .and(warp::ws())
        .and(map_user_senderchannel)
        .and(map_user_playerstate)
        .map(|ws: warp::ws::Ws, users, playerstates| {
            ws.on_upgrade(move |socket| user_connected(socket, users, playerstates))
        });

    // Static route to get server status
    let status = warp::path!("status").map(move || warp::reply::html("hello"));

    // Combining the old status route with the new one
    let routes = status.or(game);

    // Launch server on spcified port
    let port = 3030;
    log::info!("Starting server at port: {}", port);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
