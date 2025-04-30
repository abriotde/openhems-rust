use std::collections::HashMap;
use std::fmt;
use yaml_rust2::Yaml;
use crate::node;
use crate::home_assistant_api::HomeStateUpdater;
use crate::home_assistant_api::HomeAssistantAPI;



#[derive(fmt::Debug)]
pub struct Network<Updater:HomeStateUpdater> {
    network_updater: Updater,
    nodes: Vec<Box<dyn node::Node>>,
    filtered_nodes_cache: HashMap<String, Vec<Box<dyn node::Node>>>,
    margin_power_on: f32,
	margin_power_on_loop_nb: u32,
	server: u64
}
impl<Updater:HomeStateUpdater+fmt::Display> fmt::Display for Network<Updater> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network({})", self.network_updater)
    }
}

pub fn get<Updater:HomeStateUpdater+fmt::Display>(mut network_updater:Updater, nodes_conf:Vec<&Yaml>) -> Network<Updater> {
	let nodes = network_updater.get_nodes(nodes_conf);
	let network = Network {
		network_updater: network_updater,
		nodes: nodes,
		filtered_nodes_cache: HashMap::new(),
		margin_power_on: -1.0,
		margin_power_on_loop_nb: 0,
		server: 0
	};
	println!("Network({network})");
	network
}

impl<Updater:HomeStateUpdater> Network<Updater> {
	fn get_margin_power(&mut self, nodes_conf:HashMap<String, &Yaml>) {
	}
}