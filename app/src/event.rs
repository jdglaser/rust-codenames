use actix::{Addr, Message};
use serde::{Deserialize, Serialize};

use crate::{
    client::{WsClient, ClientSession},
    database::Database,
    game::{Card, Game, Team},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Connect { id: usize },
    SetName { id: usize, name: String },
    Disconnect { id: usize },
    TimedOut { id: usize },
    #[serde(rename_all = "camelCase")]
    Message {
        sender: ClientSession,
        text: String,
    },
    // Game events
    #[serde(rename_all = "camelCase")]
    FlipCard { flipped_card: Card },
    NewGame {},
    GameStateUpdate { game: Game },
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub sender: ClientSession,
    pub room: String,
    pub event: Event,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum ClientRequestType {
    Connect { id: usize },
    SetName { name: String },
    Disconnect { id: usize },
    TimedOut { id: usize },
    Message { text: String },
    FlipCard { coord: (usize, usize) },
    NewGame {},
    GameOver { winning_team: Team, reason: String },
    Health {}
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub struct ClientRequest {
    pub sender_id: usize,
    pub room_name: String,
    pub request: ClientRequestType,
}

#[derive(Message)]
#[rtype("usize")]
pub struct NewClientConnection<T: 'static + Database + std::marker::Unpin> {
    pub room: String,
    pub addr: Addr<WsClient<T>>,
}
