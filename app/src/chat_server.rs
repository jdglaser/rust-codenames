use std::collections::{HashMap, HashSet};

use actix::{Running, Message, Actor, Addr, Context, AsyncContext, Handler, StreamHandler, WrapFuture, ActorFutureExt, fut, ActorContext, ContextFutureSpawner};
use actix_web_actors::ws;
use rand::{rngs::ThreadRng, Rng};
use log::info;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
struct ClientMessage {
    sender_id: usize,
    msg: String,
    room: String,
}

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
    pub fn new() -> Self {
        ChatServer { rooms: HashMap::new(), rng: rand::thread_rng() }
    }

    fn send_msg(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for (id, addr) in sessions {
                if *id != skip_id {
                    info!("Sending message to id {} with value {}", *id, message);
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
        self.rooms.entry(msg.room.clone())
            .or_insert_with(|| HashMap::new())
            .insert(id, msg.addr.clone());
        
        info!("Added session id {} to room {}", id, msg.room);

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

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        info!("Sending message from session {} to room {}", msg.sender_id, msg.room);
        self.send_msg(&msg.room, &msg.msg, msg.sender_id);
    }
}

pub struct ChatSession {
    id: usize,
    server: Addr<ChatServer>,
    room: String,
}

impl ChatSession {
    pub fn new(server: Addr<ChatServer>, room: String) -> Self {
        ChatSession { id: 0, server, room }
    }
}

/// Make actor from `ChatSession`
impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.server
            .send(Connect { room: self.room.clone(), addr: ctx.address() } )
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(id) => act.id = id,
                    _ => ctx.stop(),
                };
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.server.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<ServerMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => self.server.do_send(ClientMessage {
                sender_id: self.id,
                msg: text.to_string(),
                room: self.room.clone(),
            }),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}