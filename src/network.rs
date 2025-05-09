use core::net;
use std::collections::HashMap;
use yaml_rust2::Yaml;
use std::fmt;
use crate::configuration_manager::ConfigurationManager;
use crate::contract::Contract;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::node::{self, Node, PublicPowerGrid, Switch};
use crate::home_assistant_api::{HomeStateUpdater,HomeAssistantAPI};
use crate::cast_utility;
use crate::server::Server;
use crate::time::HoursRanges;

// Rust equivalent of Python nodes list with multi nodes types.
#[derive(Clone, Debug)]
pub struct NodesHeap<'a> {
	publicpowergrid: Option<node::PublicPowerGrid<'a>>,
	switch: Vec<node::Switch<'a>>,
	// solarpanel: Vec<node::SolarPanel>,
	// battery: Vec<node::Battery>,
}
struct NodesHeapIterator<'a> {
	nodetype: node::NodeType,
	index:usize,
	filter: String,
	heap: &'a NodesHeap<'a>
}
/* impl<'a> fmt::Debug for NodesHeap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		let mut iter= self.get_all();
		while let Some(node) = iter.next() {
			let _ = write!(f, ", {}", node);
		}
		Ok(())
    }
} */

/* impl<'a> NodesHeapIterator<'a, 'a> {
    fn next(&'a mut self) -> Option<Box<&'a dyn Node>> {
		match self.nodetype {
			node::NodeType::PublicPowerGrid => {
				if let Some(power) = self.heap.publicpowergrid.as_ref() {
					self.index = 0;
					self.nodetype = node::NodeType::Switch;
					Some(Box::new(power))
				} else {
					self.index = 0;
					self.nodetype = node::NodeType::Switch;
					self.next()
				}
			}
			node::NodeType::Switch => {
				if self.index<self.heap.switch.len() {
					self.index += 1;
					Some(Box::new(&self.heap.switch[self.index-1]))
				} else {
					self.index = 0;
					self.nodetype = node::NodeType::NodeBase;
					self.next()
				}
			}
			_ => {
				None
			}
		}
    }
} */
impl<'a> Iterator for NodesHeapIterator<'a> {
    type Item = Box<&'a dyn Node>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.nodetype {
			node::NodeType::PublicPowerGrid => {
				if let Some(power) = self.heap.publicpowergrid.as_ref() {
					self.index = 0;
					self.nodetype = node::NodeType::Switch;
					Some(Box::new(power))
				} else {
					self.index = 0;
					self.nodetype = node::NodeType::Switch;
					self.next()
				}
			}
			node::NodeType::Switch => {
				if self.index<self.heap.switch.len() {
					self.index += 1;
					Some(Box::new(&self.heap.switch[self.index-1]))
				} else {
					self.index = 0;
					self.nodetype = node::NodeType::NodeBase;
					self.next()
				}
			}
			_ => {
				None
			}
		}
    }
}

impl<'a, 'b:'a> NodesHeap<'a> {
	pub fn new() -> NodesHeap<'a> {
		NodesHeap {
			publicpowergrid: None,
			switch: Vec::new(),
			// solarpanel: Vec<node::SolarPanel>,
			// battery: Vec<node::Battery>,
		}
	}
	pub fn get_all(&'b self) -> NodesHeapIterator<'a>
	{
		NodesHeapIterator {
			nodetype: node::NodeType::PublicPowerGrid,
			index: 0,
			filter: "all".to_string(),
			heap: self,
		}
	}
	pub fn set_switch(&mut self, network:&'b Network<'a, 'a>, nameid:&str, updater:&'a HomeAssistantAPI, node_conf:&HashMap<String, &Yaml>) -> ResultOpenHems<()> {
		// println!("HA:get_switch({nameid})");
		let priority = updater.get_feeder_const_int(node_conf, "priority", 50);
		let strategy_nameid = updater.get_feeder_const_str(node_conf, "strategy", "default");
		let base = updater.get_nodebase(network, nameid, node_conf)?;
		let switch = node::get_switch(base, priority as u32, &strategy_nameid)?;
		self.switch.push(switch);
		Ok(())
	}
	pub fn set_publicpowergrid(&mut self, network:&'b Network<'a, 'a>, nameid:&str, updater:&'a HomeAssistantAPI, node_conf:&HashMap<String, &Yaml>)  -> ResultOpenHems<()> {
		let base = updater.get_nodebase(network, nameid, node_conf)?;
		if let Some(contract_conf) = node_conf.get("contract") {
			let contract = Contract::get_from_conf(contract_conf)?;
			let node = node::get_publicpowergrid(base, contract)?;
			self.publicpowergrid = Some(node);
			Ok(())
		} else {
			Err(OpenHemsError::new(format!(
				"No key 'contract' in publicpowergrid configuration."
			)))
		}
	}
	fn get_publicpowergrid(&self) -> & Option<node::PublicPowerGrid<'a>> {
		& self.publicpowergrid
	}
	fn get_all_switch(&self, _pattern:&str) -> &Vec<node::Switch<'a>> {
		& self.switch
	}
}

#[derive(Clone)]
pub struct Network<'a, 'b:'a> {
    updater: HomeAssistantAPI<'a, 'a>,
    nodes: NodesHeap<'a>,
    margin_power_on: f32,
	margin_power_on_cache_id: u32,
	server: Option<&'b Server<'a, 'a>>
}
/* impl<'a, 'b:'a> fmt::Display for Network<'a, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{}> (\n", self.updater)?;
        for node in self.nodes.get_all() {
			write!(f, " - {}\n", node)?;
		}
		write!(f, ")")
    }
} */

impl<'a, 'b:'a, 'c:'b> Network<'a, 'a>
{
	pub fn set_server(&mut self, server:&'b Server<'a, 'a>) {
		self.server = Some(server);
	}
	pub fn new(configurator:&'a ConfigurationManager) -> ResultOpenHems<Self> {
		let margin_power_on = 0.0;
		let margin_power_on_cache_id = 0;
		let network_source = configurator.get_as_str("server.network");
		let updater:HomeAssistantAPI;
		let mut nodes = NodesHeap::new();
		match network_source.as_str() {
			"homeassistant" => {
				// println!("Network: HomeAssistantAPI");
				updater = HomeAssistantAPI::new(configurator)?;
			}
			"fake" => {
				println!("TODO : Network: FakeNetwork");
				// let network_updater = FakeNetwork(configurator)
				todo!()
			}
			_ => {
				return Err(OpenHemsError::new("Invalid server.network configuration '{networkSource}'".to_string()));
			}
		}
		let mut network = Network {
			updater: updater,
			nodes: NodesHeap::new(),
			margin_power_on: margin_power_on,
			margin_power_on_cache_id: margin_power_on_cache_id,
			server: None
		};
		Ok(network)
	}
	pub fn get_nodes(&'b self, configurator:&ConfigurationManager) -> ResultOpenHems<NodesHeap<'a>>{
		let nodes_conf = configurator.get_as_list("network.nodes");
		let count = 0;
		let mut nodes= self.nodes.clone();
		for node_c in nodes_conf {
			let node_conf: HashMap<String, &Yaml> = cast_utility::to_type_dict(node_c);
			if let Some(class) = node_conf.get("class") {
				let classname = cast_utility::to_type_str(class);
				let mut nameid: String;
				if let Some(id) = node_conf.get("id") {
					nameid = cast_utility::to_type_str(id);
				} else {
					nameid = String::from("node_");
					nameid.push_str(&count.to_string());
				}
				match &*classname.to_lowercase() {
					"switch" => {
						if let Err(err) = nodes.set_switch(self, nameid.as_str(), &self.updater, &node_conf) {
							let message = format!("Impossible to add switch '{nameid}' due to {}.", err.message);
							log::error!("ERROR {}",&message);
						}
					},
					"publicpowergrid" => {
						if let Err(err) = nodes.set_publicpowergrid(self, nameid.as_str(), &self.updater, &node_conf) {
							let message = format!("Impossible to add PublicPowerGrid '{nameid}' due to {}.", err.message);
							log::error!("ERROR {}",&message);
						}
					},
					_ => {
						let message = format!("Unknwon class '{classname}'");
						log::error!("ERROR {}",&message);
					}
				}
			} else {
				log::error!("Missing classname for node.");
			}
		}
		Ok(nodes)
	}
	pub fn get_hours_ranges(&self) -> ResultOpenHems<&HoursRanges> {
		if let Some(power) = self.nodes.get_publicpowergrid() {
			let hoursranges = power.get_contract().get_hoursranges();
			Ok(hoursranges)
		} else {
			Err(OpenHemsError::new("No public power grid.".to_string()))
		}
	}
}
