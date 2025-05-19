use log;
use chrono::{self};
use rocket::tokio;
use schedule::Schedule;
use std::time::Duration;
use env_logger;
use server::Server;
use std::{io::Write, os::unix::thread, thread::sleep, thread::spawn};
// use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::mpsc::channel;
use axum::{Router, routing::get};
use std::net::SocketAddr;

mod utils;
mod home_assistant_api;
mod cast_utility;
mod configuration_manager;
mod node;
mod network;
mod error;
mod  feeder;
mod time;
mod offpeak_strategy;
mod contract;
mod server;
mod schedule;
mod web;

fn start_web_server() {
    // Spawn a new thread to run the web server
    std::thread::spawn(|| {
        // Create a new Tokio runtime for this thread
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let app = Router::new().route("/", get(web::hello_world));
            let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    });
}

fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
               chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Debug)
        .init();
    log::info!("log level:");
	let mut configurator = configuration_manager::get(None);
	let file_path = "./config/openhems.yaml";
	if let Err(err) = configurator.add_yaml_config(file_path, false) {
		log::error!("Fail load configuration {file_path}: {err}");
	}
	let file_path = "./config/openhems.secret.yaml";
	if let Err(err) = configurator.add_yaml_config(file_path, false) {
		log::error!("Fail load configuration {file_path} : {err}");
	}
	match Server::new(&configurator) {
		Err(err) =>  {
			log::error!("Fail configure server : {}", err.message);
		}
		Ok(mut hems_server) => {
			if let Err(err) = hems_server.init(&configurator) {
				log::error!("Fail init server : {}", err.message);
			}
			log::info!("Server : {:?}", hems_server);
			let (tx, rx) = channel::<Schedule>();
			start_web_server();
			hems_server.run();
		}
	}
}
