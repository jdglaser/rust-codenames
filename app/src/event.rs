use actix::{Message, Addr};
use serde::{Serialize, Deserialize};

use crate::{client::WsClient, game::Game};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Connect {id: usize, game: Game},
    Disconnect {id: usize},
    TimedOut {id: usize},
    #[serde(rename_all = "camelCase")]
    Message {sender_id: usize, text: String}
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub room: String,
    pub event: Event
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    Message {text: String}
}

#[derive(Message)]
#[rtype("usize")]
pub struct NewClientMessage {
    pub room: String,
    pub addr: Addr<WsClient>
}