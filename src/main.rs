use actix_files as fs;
use serde::ser;
use std::sync::{Arc, Mutex};
use log;
use chrono::{self};
use schedule::Schedule;
use tera::Tera;
use web::AppState;
use env_logger;
use server::Server;
use std::{io::Write};
// use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::mpsc::channel;
use actix_web::{App, HttpServer};

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


fn start_web_server(shared_state: Arc<AppState>) -> std::thread::JoinHandle<()> {
    let tera = Tera::new("templates/**/*.jinja2")
        .expect("Failed to parse templates");
    std::thread::spawn(|| {
        // Create an Actix runtime in the new thread
        let sys = actix_rt::System::new();
        sys.block_on(async {
            let server = HttpServer::new(move || {
                App::new()
					.service(fs::Files::new("/js", "./js").show_files_listing())
					.service(fs::Files::new("/css", "./css").show_files_listing())
					.service(fs::Files::new("/img", "./img").show_files_listing())
          			.app_data(actix_web::web::Data::new(tera.clone()))
					.app_data(actix_web::web::Data::new(shared_state.clone()))
					.route("/", actix_web::web::get().to(web::index))
					.route("/states", actix_web::web::post().to(web::states))
				})
    			.workers(1)
				.bind("127.0.0.1:8000")
				.unwrap()
				.run();
			// // Listen for Ctrl+C in the Actix runtime thread
			// let server_handle = server.handle();
			// actix_web::rt::spawn(async move {
			// 	actix_web::rt::signal::ctrl_c().await.unwrap();
			// 	server_handle.stop(true).await;
			// 	panic!("Server stopped");
			// });
			server.await.unwrap();
        });
    })
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
	let mut appstate = AppState::new();
	match Server::new(&configurator) {
		Err(err) =>  {
			log::error!("Fail configure server : {}", err.message);
		}
		Ok(mut hems_server) => {
			if let Err(err) = hems_server.init(&configurator, &mut appstate) {
				log::error!("Fail init server : {}", err.message);
			}
			let appstate2 = Arc::new(appstate);
			log::info!("Server : {:?}", hems_server);
			let _ = hems_server.network.borrow().notify("OpenHEMS started");	
			let httpserver = start_web_server(Arc::clone(&appstate2));
			hems_server.run(appstate2);
			httpserver.join().unwrap();
		}
	}
}
