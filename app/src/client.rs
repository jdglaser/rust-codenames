use std::{time::{Duration, Instant}};

use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{self, WebsocketContext};
use log::{warn};
use serde::{Serialize, Deserialize};
use rand::{Rng};

use crate::{
    database::Database,
    event::{ClientRequest, ClientRequestType, EventMessage, NewClientConnection},
    server::WsServer,
};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct ClientSession {
    pub id: usize,
    pub username: String,
    pub room: String,
    pub is_spymaster: bool,
}

impl ClientSession {
    pub fn new(id: usize, room: &String) -> ClientSession {
        ClientSession {
            id,
            username: String::from(""),
            room: room.clone(),
            is_spymaster: false
        }
    }
}

pub struct WsClient<T: 'static + Database + std::marker::Unpin> {
    session_id: usize,
    room_name: String,
    database: T,
    server: Addr<WsServer<T>>,
    pub hb: Instant,
}

impl<T: 'static + Database + std::marker::Unpin> WsClient<T> {
    pub fn new(server: Addr<WsServer<T>>, room: &String, database: T) -> Self {
        WsClient {
            session_id: 0,
            server,
            database,
            room_name: room.clone(),
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                warn!("Session id {} timed out. Disconnecting.", act.session_id);

                // notify chat server
                act.server.do_send(ClientRequest {
                    sender_id: act.session_id,
                    room_name: act.room_name.clone(),
                    request: ClientRequestType::TimedOut { id: act.session_id },
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
impl<T: 'static + Database + std::marker::Unpin> Actor for WsClient<T> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start the heartbeat process
        self.hb(ctx);

        self.server
            .send(NewClientConnection {
                room: self.room_name.clone(),
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(id) => act.session_id = id,
                    _ => ctx.stop(),
                };
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.server.do_send(ClientRequest {
            sender_id: self.session_id,
            room_name: self.room_name.clone(),
            request: ClientRequestType::Disconnect { id: self.session_id },
        });
        Running::Stop
    }
}

impl<T: 'static + Database + std::marker::Unpin> Handler<EventMessage> for WsClient<T> {
    type Result = ();

    fn handle(&mut self, event_message: EventMessage, ctx: &mut Self::Context) -> Self::Result {
        let EventMessage { event, room } = event_message;

        ctx.text(serde_json::to_string(&event).unwrap())
    }
}

impl<T: 'static + Database + std::marker::Unpin> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient<T> {
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
                    sender_id: self.session_id,
                    room_name: self.room_name.clone(),
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
