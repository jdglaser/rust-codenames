use actix_web::FromRequest;
use actix::{Actor, StreamHandler};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, HttpRequest, Responder, middleware::Logger, Result, Error};
use actix_web::http::header::{ContentDisposition, DispositionType, ContentType, LOCATION, HeaderValue};
use actix_files as fs;
use std::io::Read;
use std::{path::{PathBuf, Path}, fmt::format};
use log::{info, debug, trace, error, warn};
use serde::{Serialize, Deserialize};
use actix_web_actors::ws;

mod chat_server;

use chat_server::{ChatServer};

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    age: u8,
    weight: f64
}

/// Define HTTP actor
struct User;

impl Actor for User {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for User {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                info!("Got a text message");
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (println!("uh oh")),
        }
    }
}

async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    info!("HERE");
    let room = req.match_info().query("room");
    let resp = ws::start(User {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[get("/items")]
async fn get_items() -> impl Responder {
    let items = vec![
        Item {
            name: "Bob".to_string(),
            age: 15,
            weight: 163.18
        }
    ];

    HttpResponse::Ok().json(items)
}

async fn index(req: HttpRequest) -> Result<fs::NamedFile> {
    let path_str = format!(
        "../frontend/dist/{}",
        req.match_info().query("filename")
    );
    let path: PathBuf = path_str.parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting!");

    HttpServer::new(
        || App::new()
            .service(
                web::scope("/api")
                    .service(get_items)
            )
            .service(
                web::scope("/ws")
                    .route("/{room}", web::get().to(ws_index))
            )
            .service(fs::Files::new("/", "../frontend/dist").index_file("index.html"))
            .route("/{filename:.*}", web::get().to(index))
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}