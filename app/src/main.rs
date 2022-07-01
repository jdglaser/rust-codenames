use actix::{Actor, Addr};
use actix_files as fs;
use actix_web::{
    dev::Server, get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web_actors::ws;
use database::{Database, MemoryDatabase};
use log::info;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

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
struct AppData<T: 'static + Database> {
    server: Addr<WsServer<T>>,
    database: Arc<T>,
}

async fn ws_index<T: Database>(
    path: web::Path<String>,
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppData<T>>,
) -> Result<HttpResponse, Error> {
    let room = path.into_inner();
    let resp = ws::start(
        WsClient::new(data.get_ref().server.clone(), room.to_string()),
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

async fn index(req: HttpRequest) -> Result<fs::NamedFile> {
    let path_str = format!("../frontend/dist/{}", req.match_info().query("filename"));
    let path: PathBuf = path_str.parse().unwrap();
    Ok(fs::NamedFile::open(path)?)
}

async fn create_server<T: 'static + Database + Sync + Send>(
    app_data: web::Data<AppData<T>>,
) -> Result<Server, Error> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(web::scope("/api").service(get_items))
            .service(web::scope("/ws").route("/{room}", web::get().to(ws_index::<T>)))
            .service(fs::Files::new("/", "../frontend/dist").index_file("index.html"))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .workers(4)
    .run();
    Result::Ok(server)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    info!("Starting!");

    let memory_database = Arc::new(MemoryDatabase::new());
    let chat_server = WsServer::new(memory_database.clone()).start();

    let app_data = web::Data::new(AppData {
        server: chat_server,
        database: memory_database,
    });

    create_server(app_data).await.unwrap().await
}
