use actix::{Message, Addr};
use serde::{Serialize, Deserialize};

use crate::client::ChatClient;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    Connect {id: usize},
    Disconnect {id: usize},
    Message {sender_id: usize, text: String}
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype("()")]
pub struct EventMessage {
    pub room: String,
    pub event: Event
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    Message {text: String}
}

#[derive(Message)]
#[rtype("usize")]
pub struct NewClientMessage {
    pub room: String,
    pub addr: Addr<ChatClient>
}