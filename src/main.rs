use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};

use std::path::PathBuf;




fn index(_req: HttpRequest) -> Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("~/Code/rust/fidelitas/index.html").parse().unwrap();
    let path: PathBuf = PathBuf::from("index.html");
    Ok(NamedFile::open(path)?)
}

fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


fn main() {
    let port = "8088";
    println!("Listening on port: {}...", port);


    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/hello", web::get().to(hello))
    })
    .bind(format!("127.0.0.1:{}", port))
    .unwrap()
    .run()
    .unwrap();
}
