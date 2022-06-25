use std::{collections::{HashMap, HashSet}, borrow::BorrowMut};

use actix::{Addr, Actor, Context, Handler};
use log::{info, debug};
use rand::{prelude::ThreadRng, Rng};

use crate::{client::WsClient, event::{Event, NewClientMessage, EventMessage, self}};
use crate::game::{Game};

pub struct WsServer {
    clients: HashMap<usize, Addr<WsClient>>,
    rooms: HashMap<String, Game>,
    rng: ThreadRng
}

impl WsServer {
    pub fn new() -> Self {
        WsServer { clients: HashMap::new(), rooms: HashMap::new(), rng: rand::thread_rng() }
    }

    fn send_event(&mut self, event_message: EventMessage) {
        let EventMessage { ref room, ref event } = event_message;
        if let Some(game) = self.rooms.get_mut(room) {
            if let Event::Disconnect {id} = event {
                game.remove_player(id);
            }

            for id in game.get_sessions() {
                debug!("Sending event to id {} with value {:?}", id, event);
                self.clients.get(id).unwrap().do_send(event_message.clone());
            }
        }

        if let Some(game) = self.rooms.get(room) {
            if game.get_sessions().len() == 0 {
                info!("There are no players left in room {}. Removing.", &room);
                self.rooms.remove(room);
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<NewClientMessage> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: NewClientMessage, ctx: &mut Self::Context) -> Self::Result {
        // Generate thread safe random session id
        let id = self.rng.gen::<usize>();

        // Insert the new client id to the clients
        self.clients.insert(id, msg.addr.clone());

        // Insert room if it does not exist and add session actor's address
        let game = self.rooms.entry(msg.room.clone())
            .or_insert_with(|| Game::new())
            .add_player(id)
            .clone();
        
        info!("Added session id {} to room {}", id, msg.room);

        self.send_event(EventMessage {room: msg.room.clone(), event: Event::Connect { id, game }});

        id
    }
}

impl Handler<EventMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, event_message: EventMessage, ctx: &mut Self::Context) -> Self::Result {
        self.send_event(event_message);
    }
}