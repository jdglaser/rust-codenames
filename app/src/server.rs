use std::{
    collections::{HashMap},
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, Context, Handler};
use log::{debug, info};

use crate::{game::{Game, Team}, client::ClientSession};
use crate::{
    client::WsClient,
    database::Database,
    event::{ClientRequest, ClientRequestType, Event, EventMessage, NewClientConnection}
};

#[derive(Debug, Clone, PartialEq)]
pub struct Room {
    pub name: String,
    pub game_id: usize,
    pub sessions: Vec<usize>,
}

impl Room {
    pub fn new(name: String, game_id: usize) -> Room {
        Room {
            name,
            game_id,
            sessions: Vec::new(),
        }
    }
}

pub struct WsServer<T: 'static + Database + std::marker::Unpin> {
    database: T,
    clients: HashMap<usize, Addr<WsClient<T>>>,
}

impl<T: 'static + Database + std::marker::Unpin> WsServer<T> {
    pub fn new(database: T) -> Self {
        WsServer {
            database,
            clients: HashMap::new()
        }
    }

    fn send_event(&mut self, client_request: ClientRequest) {
        let ClientRequest {
            ref sender_id,
            ref room_name,
            request,
        } = client_request;

        let room = self.database.get_room(room_name).unwrap();
        let game = self.database.get_game(room.game_id).unwrap();
        let sessions = self.database.get_room(room_name).unwrap().sessions;

        let send_message_to_clients = |event: Event| {
            for id in &sessions {
                debug!("Sending event to id {} with value {:?}", id, &event);
                self.clients.get(&id).unwrap().do_send(EventMessage {
                    room: room_name.clone(),
                    event: event.clone(),
                });
            }
        };

        let send_game_state_update_to_clients = |game: &Game| {
            debug!("Sending game state update event to room {}.", &room_name);
            for id in &sessions {
                debug!("Sending game state update event to id {}.", id);
                self.clients.get(id).unwrap().do_send(EventMessage {
                    room: room_name.clone(),
                    event: Event::GameStateUpdate { game: game.clone() },
                });
            }
        };

        match request {
            ClientRequestType::Connect { id } => {
                debug!("{} connected", id);
                send_message_to_clients(Event::Connect { id });
                send_game_state_update_to_clients(&game);
            },
            ClientRequestType::SetName { name } => {
                let existing_session = self.database.get_session(sender_id).unwrap();
                let new_session = ClientSession { username: name.clone(), ..existing_session };
                self.database.update_session(*sender_id, &new_session).unwrap();
                send_message_to_clients(Event::SetName { id: *sender_id, name });
            }
            ClientRequestType::Disconnect { id } => {
                debug!("{} disconnected.", id);
                self.database.remove_session(id).unwrap();
                if self.database.get_room(room_name).unwrap().sessions.len() == 0 {
                    info!("There are no players left in room {}. Removing.", room_name);
                    self.database.remove_room(room_name).ok();
                    return;
                }
                send_message_to_clients(Event::Disconnect { id });
                send_game_state_update_to_clients(&game);
            }
            ClientRequestType::TimedOut { id } => {
                self.database.remove_session(id).unwrap();
                if self.database.get_room(room_name).unwrap().sessions.len() == 0 {
                    info!("There are no players left in room {}. Removing.", room_name);
                    self.database.remove_room(room_name).unwrap();
                    return;
                }
                send_message_to_clients(Event::Disconnect { id });
                send_game_state_update_to_clients(&game);
            }
            ClientRequestType::Message { text } => {
                let sender_client_session = self.database.get_session(sender_id).unwrap();
                send_message_to_clients(Event::Message {
                    sender: sender_client_session,
                    text,
                });
            }
            ClientRequestType::FlipCard { coord } => {
                let new_board = game.flip_card(coord);
                let new_event = Event::FlipCard {
                    flipped_card: new_board[coord.0][coord.1].clone(),
                };
                let new_game = Game { 
                    board: new_board,
                    turn_team: Team::opposite(&game.turn_team),
                    ..game.clone() 
                };
                self.database.update_game(room.game_id, &new_game).unwrap();
                send_message_to_clients(new_event);
                send_game_state_update_to_clients(&new_game);
            }
            ClientRequestType::NewGame {} => {
                let new_game = game.new_from_current_game();
                self.database.update_game(room.game_id, &new_game).unwrap();
                send_message_to_clients(Event::NewGame {});
                send_game_state_update_to_clients(&new_game);
            }
        }
    }
}

/// Make actor from `ChatServer`
impl<T: 'static + Database + std::marker::Unpin> Actor for WsServer<T> {
    type Context = Context<Self>;
}

impl<T: 'static + Database + std::marker::Unpin> Handler<NewClientConnection<T>> for WsServer<T> {
    type Result = usize;

    fn handle(&mut self, msg: NewClientConnection<T>, ctx: &mut Self::Context) -> Self::Result {
        if self.database.get_room(&msg.room).is_err() {
            self.database.create_room(&msg.room).unwrap();
        }

        let session_id = self.database.create_session(&msg.room).unwrap();

        self.clients.insert(session_id, msg.addr);

        self.send_event(ClientRequest {
            sender_id: session_id,
            room_name: msg.room.clone(),
            request: ClientRequestType::Connect { id: session_id },
        });

        session_id
    }
}

impl<T: 'static + Database + std::marker::Unpin> Handler<ClientRequest> for WsServer<T> {
    type Result = ();

    fn handle(&mut self, msg: ClientRequest, ctx: &mut Self::Context) -> Self::Result {
        self.send_event(msg);
    }
}
