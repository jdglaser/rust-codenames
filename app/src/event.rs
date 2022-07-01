use actix::{Addr, Message};
use serde::{Deserialize, Serialize};

use crate::{
    client::WsClient,
    database::Database,
    game::{Card, Game},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Connect {
        id: usize,
    },
    Disconnect {
        id: usize,
    },
    TimedOut {
        id: usize,
    },
    #[serde(rename_all = "camelCase")]
    Message {
        sender_id: usize,
        text: String,
    },
    // Game events
    #[serde(rename_all = "camelCase")]
    FlipCard {
        flipped_card: Card,
    },
    NewGame {},
    GameStateUpdate {
        game: Game,
    },
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub room: String,
    pub event: Event,
}

impl EventMessage {
    pub fn set_event(&mut self, event: Event) {
        self.event = event;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum ClientRequestType {
    Connect { id: usize },
    Disconnect { id: usize },
    TimedOut { id: usize },
    Message { text: String },
    FlipCard { coord: (usize, usize) },
    NewGame {},
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub struct ClientRequest {
    pub sender_id: usize,
    pub room: String,
    pub request: ClientRequestType,
}

#[derive(Message)]
#[rtype("usize")]
pub struct NewClientConnection<T: 'static + Database> {
    pub room: String,
    pub addr: Addr<WsClient<T>>,
}
