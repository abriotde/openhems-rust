use log;
use chrono;
use env_logger;
use server::Server;
use std::io::Write;

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
			hems_server.run();
		}
	}
}
