use datetime::LocalDateTime;
use error::{ResultOpenHems, OpenHemsError};
use home_assistant_api::HomeAssistantAPI;
use network::Network;
use log;
use chrono;
use env_logger;
use server::Server;
use std::{io::Write, rc::Rc};

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
	match Network::new(&configurator) {
		Ok(network) => {
			let nodes = network.get_nodes(&configurator);
			let mut network2 = network.clone();
			network2.set_nodes(nodes);
			println!("{}", network);
			match Server::new(&network, &configurator) {
				Ok(mut hems_server) => {
					network2.set_server(Rc::new(&hems_server));
					let now = LocalDateTime::now();
					hems_server.loop1(now);
				}
				Err(err) => {
					log::error!("Fail load Network : {}", err.message)
				}
			}
		}
		Err(err) => {
			log::error!("Fail load Network : {}", err.message)
		}
	}
}
