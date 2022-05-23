use std::collections::{HashMap, HashSet};

use actix::{Message, Actor, Addr, Context, AsyncContext, Handler};
use actix_web_actors::ws;
use rand::{rngs::ThreadRng, Rng};
use log::info;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

#[derive(Message)]
#[rtype(result = "usize")]
struct Connect {
    room: String,
    addr: Addr<ChatSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    id: usize
}

pub struct ChatServer {
    rooms: HashMap<String, HashMap<usize, Addr<ChatSession>>>,
    rng: ThreadRng
}

impl ChatServer {
    fn send_msg(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for (id, addr) in sessions {
                if *id != skip_id {
                    addr.do_send(ServerMessage(message.to_owned()));
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        // Generate thread safe random session id
        let id = self.rng.gen::<usize>();

        // Insert room if it does not exist and add session actor's address
        self.rooms.entry(msg.room)
            .or_insert_with(|| HashMap::new())
            .insert(id, msg.addr);

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        let mut left_rooms: Vec<String> = Vec::new();

        for (room, sessions) in &mut self.rooms {
            if sessions.remove(&msg.id).is_some() {
                info!("Removed session id {} from room {}", msg.id, room);
                left_rooms.push(room.to_owned());
            }
        }

        for room in left_rooms {
            self.send_msg(&room, &format!("{} diconnected from room {}", msg.id, room), 0);
        }
    }
}

pub struct ChatSession {
    id: usize,
    server: Addr<ChatServer>,
    rooms: HashMap<String, HashSet<usize>>,
}

/// Make actor from `ChatSession`
impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
    }

}

impl Handler<ServerMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}