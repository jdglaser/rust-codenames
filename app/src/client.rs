use std::time::{Duration, Instant};

use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{self, WebsocketContext};
use log::warn;

use crate::{
    database::Database,
    event::{ClientRequest, ClientRequestType, EventMessage, NewClientConnection},
    server::WsServer,
};

pub struct WsClient<T: 'static + Database> {
    id: usize,
    server: Addr<WsServer<T>>,
    room: String,
    /// Client must send ping at least once per 10 seconds, otherwise we drop connection.
    pub hb: Instant,
}

impl<T: Database> WsClient<T> {
    pub fn new(server: Addr<WsServer<T>>, room: String) -> Self {
        WsClient {
            id: 0,
            server,
            room,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Session id {} timed out. Disconnecting.", act.id);

                // notify chat server
                act.server.do_send(ClientRequest {
                    sender_id: act.id,
                    room: act.room.clone(),
                    request: ClientRequestType::TimedOut { id: act.id },
                });

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
impl<T: Database> Actor for WsClient<T> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start the heartbeat process
        self.hb(ctx);

        self.server
            .send(NewClientConnection {
                room: self.room.clone(),
                addr: ctx.address(),
            })
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
        self.server.do_send(ClientRequest {
            sender_id: self.id,
            room: self.room.clone(),
            request: ClientRequestType::Disconnect { id: self.id },
        });
        Running::Stop
    }
}

impl<T: Database> Handler<EventMessage> for WsClient<T> {
    type Result = ();

    fn handle(&mut self, event_message: EventMessage, ctx: &mut Self::Context) -> Self::Result {
        let EventMessage { event, room } = event_message;

        ctx.text(serde_json::to_string(&event).unwrap())
    }
}

impl<T: Database> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient<T> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let client_request_type: ClientRequestType = serde_json::from_str(&text).unwrap();
                self.server.do_send(ClientRequest {
                    sender_id: self.id,
                    room: self.room.clone(),
                    request: client_request_type,
                });
            }
            Ok(ws::Message::Close(_)) => {
                ctx.stop();
            }
            _ => (warn!("Did not recognize event {:?}", msg)),
        }
    }
}
