use core::net;
use std::collections::HashMap;
use std::rc::Rc;
use yaml_rust2::Yaml;
use std::fmt::{self, Display};
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
#[derive(Clone, Debug)]
struct NodesHeapIterator<'a, 'b> {
	nodetype: node::NodeType,
	index:usize,
	filter: String,
	heap: &'b NodesHeap<'a>
}
/* impl<'a> fmt::Debug for NodesHeap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		for node in self.get_all() {
			let _ = write!(f, ", {}", node);
		}
		Ok(())
    }
} */

impl<'a, 'b:'a> Iterator for NodesHeapIterator<'a, 'b> {
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
	pub fn get_all<'g>(&'g self) -> NodesHeapIterator<'g, 'g>
	{
		NodesHeapIterator {
			nodetype: node::NodeType::PublicPowerGrid,
			index: 0,
			filter: "all".to_string(),
			heap: self,
		}
	}
	pub fn set_switch(&mut self, network:&'b Network<'a>, nameid:&str, updater:&'a HomeAssistantAPI, node_conf:&HashMap<String, &Yaml>) -> ResultOpenHems<()> {
		// println!("set_switch({nameid})");
		let priority = updater.get_feeder_const_int(node_conf, "priority", 50);
		let strategy_nameid = updater.get_feeder_const_str(node_conf, "strategy", "default");
		let base = updater.get_nodebase(network, nameid, node_conf)?;
		let switch = node::get_switch(base, priority as u32, &strategy_nameid)?;
		self.switch.push(switch);
		// println!("set_switch({nameid}) : Ok");
		Ok(())
	}
	pub fn set_publicpowergrid(&mut self, network:&'b Network<'a>, nameid:&str, updater:&'a HomeAssistantAPI, node_conf:&HashMap<String, &Yaml>)  -> ResultOpenHems<()> {
		// println!("set_publicpowergrid()");
		let base = updater.get_nodebase(network, nameid, node_conf)?;
		if let Some(contract_conf) = node_conf.get("contract") {
			let contract = Contract::get_from_conf(contract_conf)?;
			let node = node::get_publicpowergrid(base, contract)?;
			self.publicpowergrid = Some(node);
			// println!("set_publicpowergrid() : Ok");
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

#[derive(Clone, Debug)]
pub struct Network<'a> {
    updater: HomeAssistantAPI<'a, 'a>,
    nodes: NodesHeap<'a>,
    margin_power_on: f32,
	margin_power_on_cache_id: u32,
	server: Option<Rc<&'a Server<'a, 'a>>>
}
impl<'a, 'b:'a> Display for Network<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{:?}> (\n{:?}\n)", self.updater, self.nodes)
    }
}

impl<'a, 'b:'a, 'c:'b> Network<'a>
{
	pub fn set_server(&mut self, server:Rc<&'b Server<'a, 'a>>) {
		self.server = Some(server);
	}
	pub fn set_nodes(&mut self, nodes:NodesHeap<'a>) {
		self.nodes = nodes;
	}
	pub fn new(configurator:&'a ConfigurationManager) -> ResultOpenHems<Self> {
		let margin_power_on = 0.0;
		let margin_power_on_cache_id = 0;
		let network_source = configurator.get_as_str("server.network");
		let updater:HomeAssistantAPI;
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
		let network = Network {
			updater: updater,
			nodes: NodesHeap::new(),
			margin_power_on: margin_power_on,
			margin_power_on_cache_id: margin_power_on_cache_id,
			server: None
		};
		Ok(network)
	}
	pub fn get_nodes(&'b self, configurator:&ConfigurationManager) -> NodesHeap<'a> {
		let nodes_conf = configurator.get_as_list("network.nodes");
		let count = 0;
		let mut nodes= NodesHeap::new();
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
		// println!("Nodes:{nodes:?}");
		nodes
	}
	pub fn get_hours_ranges(&self) -> ResultOpenHems<&HoursRanges> {
		if let Some(power) = self.nodes.get_publicpowergrid() {
			let hoursranges = power.get_contract().get_hoursranges();
			Ok(hoursranges)
		} else {
			Err(OpenHemsError::new("Need a public power grid for hours ranges but there is not.".to_string()))
		}
	}
}
