use actix_web::FromRequest;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, HttpRequest, Responder, middleware::Logger, Result};
use actix_web::http::header::{ContentDisposition, DispositionType, ContentType, LOCATION, HeaderValue};
use actix_files as fs;
use std::io::Read;
use std::{path::{PathBuf, Path}, fmt::format};
use log::{info, debug, trace, error, warn};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    age: u8,
    weight: f64
}

#[get("/items")]
async fn get_items() -> impl Responder {
    print!("HELLO!!!");
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
            .service(fs::Files::new("/", "../frontend/dist").index_file("index.html"))
            .route("/{filename:.*}", web::get().to(index))
        )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}