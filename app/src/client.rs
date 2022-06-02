use std::time::{Duration, Instant};

use actix::{Addr, Actor, fut, Running, Handler, StreamHandler, AsyncContext, WrapFuture, ActorFutureExt, ActorContext, ContextFutureSpawner};
use actix_web_actors::ws::{self, WebsocketContext};
use log::{info, error, warn, debug};

use crate::{server::ChatServer, event::{Event, NewClientMessage, EventMessage, ClientMessage}};

pub struct ChatClient {
    id: usize,
    server: Addr<ChatServer>,
    room: String,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,
}

impl ChatClient {
    pub fn new(server: Addr<ChatServer>, room: String) -> Self {
        ChatClient { id: 0, server, room, hb: Instant::now() }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Session id {} timed out. Disconnecting.", act.id);

                // notify chat server
                act.server.do_send(EventMessage { room: act.room.clone(), event: Event::TimedOut { id: act.id } });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Make actor from `ChatSession`
impl Actor for ChatClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start the heartbeat process
        self.hb(ctx);
        
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
            Event::Disconnect { id } => {
                ctx.text(format!("{} left room {}.", id, room))
            },
            Event::TimedOut { id } => ctx.text(format!("{} has timed out!", id)),
            Event::Message { sender_id, text} => ctx.text(format!("{}: {}", sender_id, text))
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            },
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            },
            Ok(ws::Message::Text(text)) => {
                let client_message: ClientMessage = serde_json::from_str(&text).unwrap();
                match client_message {
                    ClientMessage::Message { text } => self.server.do_send(EventMessage {
                        room: self.room.clone(),
                        event: Event::Message { sender_id: self.id, text }
                    })
                }
            },
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            _ => (warn!("Did not recognize event {:?}", msg)),
        }
    }
}