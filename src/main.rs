use error::{ResultOpenHems, OpenHemsError};
use home_assistant_api::HomeAssistantAPI;
use network::Network;

mod home_assistant_api;
mod cast_utility;
mod configuration_manager;
mod node;
mod network;
mod error;

fn main() {
    println!("Hello, world!");
	let mut configurator: configuration_manager::ConfigurationManager = configuration_manager::get(None);
	let file_path = "./config/openhems.yaml";
	let _ = configurator.add_yaml_config(file_path, false);
	let file_path = "./config/openhems.secret.yaml";
	let _ = configurator.add_yaml_config(file_path, false);
	let _ = get_network(&configurator);
}

fn get_network(configurator:&configuration_manager::ConfigurationManager) -> ResultOpenHems<Network<HomeAssistantAPI>> {
	let network_source = configurator.get_as_str("server.network");
	let nodes_conf = configurator.get_as_list("network.nodes");
	if network_source=="homeassistant" {
		// println!("Network: HomeAssistantAPI");
		let url = configurator.get_as_str("api.url");
		let token = configurator.get_as_str("api.long_lived_token");
		let network_updater = home_assistant_api::get(url, token);
		let mut network = network::get(network_updater);
		network.set_nodes(nodes_conf);
		println!("{}", network);
		Ok(network)
	} else { if network_source=="fake" {
		println!("TODO : Network: FakeNetwork");
		// let network_updater = FakeNetwork(configurator)
		Err(OpenHemsError::new("Un-implemented  fake network updater".to_string()))
	} else {
		Err(OpenHemsError::new("Invalid server.network configuration '{networkSource}'".to_string()))
	}}
}