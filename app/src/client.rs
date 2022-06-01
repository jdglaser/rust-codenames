use actix::{Addr, Actor, fut, Running, Handler, StreamHandler, AsyncContext, WrapFuture, ActorFutureExt, ActorContext, ContextFutureSpawner};
use actix_web_actors::ws;
use log::{info, error, warn};

use crate::{server::ChatServer, event::{Event, NewClientMessage, EventMessage, ClientMessage}};

pub struct ChatClient {
    id: usize,
    server: Addr<ChatServer>,
    room: String,
}

impl ChatClient {
    pub fn new(server: Addr<ChatServer>, room: String) -> Self {
        ChatClient { id: 0, server, room }
    }
}

/// Make actor from `ChatSession`
impl Actor for ChatClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.server
            .send(NewClientMessage { room: self.room.clone(), addr: ctx.address() } )
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
        self.server.do_send(EventMessage {room: self.room.clone(), event: Event::Disconnect { id: self.id }});
        Running::Stop
    }
}

impl Handler<EventMessage> for ChatClient {
    type Result = ();

    fn handle(&mut self, event_message: EventMessage, ctx: &mut Self::Context) -> Self::Result {
        let EventMessage {event, room} = event_message;
        match event {
            Event::Connect {id} => ctx.text(format!("{} joined room {}.", id, room)),
            Event::Disconnect { id } => ctx.text(format!("{} left room {}.", id, room)),
            Event::Message { sender_id, text} => ctx.text(format!("{}: {}", sender_id, text))
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let client_message: ClientMessage = serde_json::from_str(&text).unwrap();
                match client_message {
                    ClientMessage::Message { text } => self.server.do_send(EventMessage {
                        room: self.room.clone(),
                        event: Event::Message { sender_id: self.id, text }
                    })
                }
            },
            _ => (warn!("Did not recognize event {:?}", msg)),
        }
    }
}