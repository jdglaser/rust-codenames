use std::collections::{HashMap, HashSet};

use actix::{Addr, Actor, Context, Handler};
use log::{info, debug};
use rand::{prelude::ThreadRng, Rng};

use crate::{client::ChatClient, event::{Event, NewClientMessage, EventMessage, self}};

pub enum CardType {
    RED,
    BLUE,
    BYSTANDER,
    ASSASSIN,
}

pub struct Card {
    word: String,
    card_type: CardType,
    flipped: bool,
}

pub struct Game {
    board: [[Card; 5]; 5]
}

pub struct ChatServer {
    clients: HashMap<usize, Addr<ChatClient>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer { clients: HashMap::new(), rooms: HashMap::new(), rng: rand::thread_rng() }
    }

    fn send_event(&mut self, event_message: EventMessage) {
        let EventMessage { ref room, ref event } = event_message;
        if let Some(sessions) = self.rooms.get_mut(room) {
            if let Event::Disconnect {id} = event {
                sessions.remove(id);
            }

            for id in &*sessions {
                debug!("Sending event to id {} with value {:?}", id, event);
                self.clients.get(&id).unwrap().do_send(event_message.clone());
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<NewClientMessage> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: NewClientMessage, ctx: &mut Self::Context) -> Self::Result {
        // Generate thread safe random session id
        let id = self.rng.gen::<usize>();

        // Insert the new client id to the clients
        self.clients.insert(id, msg.addr.clone());

        // Insert room if it does not exist and add session actor's address
        self.rooms.entry(msg.room.clone())
            .or_insert_with(|| HashSet::<usize>::new())
            .insert(id);
        
        info!("Added session id {} to room {}", id, msg.room);

        self.send_event(EventMessage {room: msg.room.clone(), event: Event::Connect { id }});

        id
    }
}

impl Handler<EventMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, event_message: EventMessage, ctx: &mut Self::Context) -> Self::Result {
        self.send_event(event_message);
    }
}