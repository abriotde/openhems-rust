use actix_web::{get, web, App, HttpServer, Responder};
use rocket::routes;

// For Jinja :  tera or minijinja
/*
#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
pub async fn run_webserver() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}
*/

// #[get("/hello/<name>/<age>")]
async fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

pub async fn hello_world() -> &'static str {
    "Hello, world!"
}
