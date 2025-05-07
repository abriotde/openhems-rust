use error::{ResultOpenHems, OpenHemsError};
use home_assistant_api::HomeAssistantAPI;
use network::Network;
use log;
use chrono;
use env_logger;
use std::io::Write;

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
	let mut network: Network<'_, HomeAssistantAPI> = Network::default();
	let mut configurator: configuration_manager::ConfigurationManager = configuration_manager::get(None);
	let file_path = "./config/openhems.yaml";
	let _ = configurator.add_yaml_config(file_path, false);
	let file_path = "./config/openhems.secret.yaml";
	let _ = configurator.add_yaml_config(file_path, false);
	if let Err(message) = init_network(&mut network, &configurator) {
		log::error!("ERROR initializing network  : {message}")
	} else {

	}
}

fn init_network<'a>(network: &'a mut Network<'a, HomeAssistantAPI>, configurator:&'a configuration_manager::ConfigurationManager) -> ResultOpenHems<()> {
	let network_source = configurator.get_as_str("server.network");
	let nodes_conf: Vec<&'a yaml_rust2::Yaml> = configurator.get_as_list("network.nodes");
	if network_source=="homeassistant" {
		// println!("Network: HomeAssistantAPI");
		let url = configurator.get_as_str("api.url");
		let token = configurator.get_as_str("api.long_lived_token");
		if let Err(err) = network.updater.init(url, token) {
			return Err(err)
		}
		network.set_nodes(nodes_conf);
		// log::info!("{}", network);
		Ok(())
	} else { if network_source=="fake" {
		println!("TODO : Network: FakeNetwork");
		// let network_updater = FakeNetwork(configurator)
		Err(OpenHemsError::new("Un-implemented  fake network updater".to_string()))
	} else {
		Err(OpenHemsError::new("Invalid server.network configuration '{networkSource}'".to_string()))
	}}
}