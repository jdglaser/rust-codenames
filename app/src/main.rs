use actix::{Actor, Addr};
use actix_web::{get, web, App, HttpResponse, HttpServer, HttpRequest, Responder, Result, Error};
use actix_files as fs;
use std::{path::{PathBuf}};
use log::{info};
use serde::{Serialize, Deserialize};
use actix_web_actors::ws;

mod event;
mod server;
mod client;
mod game;
mod database;

use client::WsClient;
use server::WsServer;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    age: u8,
    weight: f64
}

async fn ws_index(path: web::Path<String>, req: HttpRequest, stream: web::Payload, data: web::Data<Addr<WsServer>>) -> Result<HttpResponse, Error> {
    let room = path.into_inner();
    let resp = ws::start(
        WsClient::new(data.get_ref().clone(), room.to_string()), &req, stream
    );
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    info!("Starting!");

    let chat_server = WsServer::new().start();

    HttpServer::new(
        move || App::new()
            .app_data(web::Data::new(chat_server.clone()))
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