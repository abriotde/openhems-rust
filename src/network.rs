use std::collections::HashMap;
use std::fmt;
use yaml_rust2::Yaml;
use crate::node;
use crate::home_assistant_api::HomeStateUpdater;



#[derive(fmt::Debug)]
pub struct Network<'a, Updater:HomeStateUpdater> {
    network_updater: Updater,
    nodes: Vec<node::Node>,
    filtered_nodes_cache: HashMap<String, Vec<&'a node::Node>>,
    margin_power_on: f32,
	margin_power_on_loop_nb: u32,
	server: u64
}
impl<'a, Updater:HomeStateUpdater+fmt::Display> fmt::Display for Network<'a, Updater> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{}> (\n", self.network_updater)?;
		for node in self.nodes.iter() {
			write!(f, " - {}\n", node)?;
		}
		write!(f, ")")
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
	println!("INFO {network}");
	network
}

impl<'a, Updater:HomeStateUpdater> Network<'a, Updater> {
	fn get_margin_power(&mut self, nodes_conf:HashMap<String, &Yaml>) {
	}
}