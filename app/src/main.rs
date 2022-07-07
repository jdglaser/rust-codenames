use actix::{Actor, Addr};
use actix_files as fs;
use actix_web::{
    dev::Server, get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_actors::ws;
use database::{Database, MemoryDatabase};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf};

mod client;
mod database;
mod event;
mod game;
mod server;

use client::WsClient;
use server::WsServer;

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    age: u8,
    weight: f64,
}

#[derive(Clone)]
struct AppData<T: 'static + Database + std::marker::Unpin> {
    server: Addr<WsServer<T>>,
    database: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    username: String
}

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Assets;

async fn ws_index<T: Database + 'static + std::marker::Unpin + Clone>(
    path: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppData<T>>
) -> Result<HttpResponse, Error> {
    let room = path.into_inner();
    let resp = ws::start(
        WsClient::new(data.get_ref().server.clone(), &room.to_string(), data.database.clone()),
        &req,
        stream,
    );
    resp
}

#[get("/items")]
async fn get_items() -> impl Responder {
    let items = vec![Item {
        name: "Bob".to_string(),
        age: 15,
        weight: 163.18,
    }];

    HttpResponse::Ok().json(items)
}

fn handle_embedded_file(path: &str) -> HttpResponse {
    println!("PATH: {}", path);
    match Assets::get(path) {
      Some(content) => HttpResponse::Ok()
        .content_type(from_path(path).first_or_octet_stream().as_ref())
        .body(content.data.into_owned()),
      None => HttpResponse::NotFound().body("404 Not Found"),
    }
  }

#[actix_web::get("/{_:.*}")]
async fn index(req: HttpRequest) -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/assets/{_:.*}")]
async fn dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(&format!("assets/{}", path.as_str()))
}

async fn create_server<T: 'static + Database + Sync + Send + std::marker::Unpin + Clone>(
    app_data: web::Data<AppData<T>>,
) -> Result<Server, Error> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(web::scope("/api").service(get_items))
            .service(web::scope("/ws").route("/{room}", web::get().to(ws_index::<T>)))
            .service(dist)
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .workers(4)
    .run();
    Result::Ok(server)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let memory_database = MemoryDatabase::new();
    let chat_server = WsServer::new(memory_database.clone()).start();

    let app_data = web::Data::new(AppData {
        server: chat_server,
        database: memory_database,
    });

    create_server(app_data)
        .await
        .unwrap()
        .await
}
