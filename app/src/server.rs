use std::{collections::{HashMap, HashSet}, borrow::BorrowMut};

use actix::{Addr, Actor, Context, Handler};
use log::{info, debug};
use rand::{prelude::ThreadRng, Rng};

use crate::{client::WsClient, event::{Event, NewClientConnection, EventMessage, self, ClientRequest, ClientRequestType}};
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

    fn send_event(&mut self, client_request: ClientRequest) {
        let ClientRequest { ref sender_id, ref room, request } = client_request;
        let game = self.rooms.get_mut(room).expect("Unable to find game");
        let sessions = game.get_sessions().clone();
        
        let send_message_to_clients = |event: Event| {
            for id in &sessions {
                debug!("Sending event to id {} with value {:?}", id, &event);
                self.clients.get(&id).unwrap().do_send(EventMessage { room: room.clone(), event: event.clone() });
            }
        };

        let send_game_state_update_to_clients = |game: &Game| {
            debug!("Sending game state update event to room {}.", &room);
            for id in &sessions {
                debug!("Sending game state update event to id {}.", id);
                self.clients.get(id).unwrap().do_send(EventMessage { room: room.clone(), event: Event::GameStateUpdate { game: game.clone() } });
            }
        };

        match request {
            ClientRequestType::Connect { id } => {
                send_message_to_clients(Event::Connect { id });
                send_game_state_update_to_clients(game);
            }
            ClientRequestType::Disconnect { id } => {
                game.remove_player(&id);
                if game.get_sessions().len() == 0 {
                    info!("There are no players left in room {}. Removing.", &room);
                    self.rooms.remove(room);
                    return;
                }
                send_message_to_clients(Event::Disconnect { id });
                send_game_state_update_to_clients(game);
            },
            ClientRequestType::TimedOut { id } => {
                game.remove_player(&id);
                if game.get_sessions().len() == 0 {
                    info!("There are no players left in room {}. Removing.", &room);
                    self.rooms.remove(room);
                    return;
                }
                send_message_to_clients(Event::Disconnect { id });
                send_game_state_update_to_clients(game);
            }
            ClientRequestType::Message { text } => {
                send_message_to_clients(Event::Message { sender_id: *sender_id, text });
            },
            ClientRequestType::FlipCard { coord } => {
                let flipped_card = game.flip_card(coord);
                let new_event = Event::FlipCard { flipped_card };
                send_message_to_clients(new_event);
                send_game_state_update_to_clients(game);
            },
            ClientRequestType::NewGame {  } => {
                let game = game.new_from_current_game();
                self.rooms.insert(room.clone(), game.clone());
                send_message_to_clients(Event::NewGame {  });
                send_game_state_update_to_clients(&game);
            },
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<NewClientConnection> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: NewClientConnection, ctx: &mut Self::Context) -> Self::Result {
        // Generate thread safe random session id
        let id = self.rng.gen::<usize>();

        // Insert the new client id to the clients
        self.clients.insert(id, msg.addr.clone());

        // Insert room if it does not exist and add session actor's address
        self.rooms.entry(msg.room.clone())
            .or_insert_with(|| Game::new())
            .add_player(id);
        
        info!("Added session id {} to room {}", id, msg.room);

        self.send_event(ClientRequest {sender_id: id, room: msg.room.clone(), request: ClientRequestType::Connect { id }});

        id
    }
}

impl Handler<ClientRequest> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: ClientRequest, ctx: &mut Self::Context) -> Self::Result {
        self.send_event(msg);
    }
}