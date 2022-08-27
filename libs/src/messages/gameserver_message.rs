use serde::{Deserialize, Serialize};

use crate::messages::remote_state::RemoteState;

use super::State;

#[derive(Deserialize, Serialize)]
pub enum GameServerMessage {
    Welcome(u16),
    Goodbye(u16),
    ServerUpdate(Vec<RemoteState>),
    ClientUpdate(State),
}
