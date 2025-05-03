use core::net;
use std::collections::HashMap;
use std::fmt;
use yaml_rust2::Yaml;
use crate::node::{self, PublicPowerGrid};
use crate::home_assistant_api::HomeStateUpdater;
use crate::cast_utility;
use crate::error::{OpenHemsError, ResultOpenHems};

// Rust equivalent of Python nodes list with multi nodes types.
#[derive(Clone)]
pub struct NodesHeap {
	publicpowergrid: Option<node::PublicPowerGrid>,
	switch: Vec<node::Switch>,
	// solarpanel: Vec<node::SolarPanel>,
	// battery: Vec<node::Battery>,
}
struct NodesHeapIterator<'a> {
	nodetype: node::NodeType,
	index:usize,
	filter: &'a str,
	heap: &'a NodesHeap
}
impl<'a> fmt::Debug for NodesHeap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for node in self.get_all("all") {
			write!(f, ", {}", node);
		}
		Ok(())
    }
}
impl<'a> Iterator for NodesHeapIterator<'a> {
    // We can refer to this type using Self::Item
    type Item = Box<dyn node::Node>;

    fn next(&mut self) -> Option<Self::Item> {
		match self.nodetype {
			node::NodeType::PublicPowerGrid => {
				if let Some(power) = self.heap.publicpowergrid {
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
					Some(Box::new(self.heap.switch[self.index-1]))
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

impl NodesHeap {
	pub fn get_all<'a>(&'a self, filter:&'a str) -> NodesHeapIterator<'a> {
		NodesHeapIterator {
			nodetype: node::NodeType::PublicPowerGrid,
			index: 0,
			filter: filter,
			heap: &self,
		}
	}
	pub fn set_node<Updater:HomeStateUpdater>(&mut self, updater:&mut Updater, classname:&str, nameid:&str, node_conf: HashMap<String, &Yaml>) -> ResultOpenHems<()> {
		match &*classname.to_lowercase() {
			"switch" => {
				let node = updater.get_switch(nameid, node_conf)?;
				self.switch.push(node);
				Ok(())
			},
			"publicpowergrid" => {
				let node = updater.get_publicpowergrid(nameid, node_conf)?;
				self.publicpowergrid= Some(node);
				Ok(())
			},
			_ => {
				let message = format!("Unknwon class '{classname}'");
				log::error!("ERROR {}",&message);
				Err(OpenHemsError::new(message))
			}
		}
	}
	pub fn set_nodes<Updater:HomeStateUpdater>(&mut self, updater:&mut Updater, nodes_conf:Vec<&Yaml>) {
		let count = 0;
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
				if let Err(error) = self.set_node(updater, &classname, &nameid, node_conf) {
					log::error!("{error}");
					// TODO : register
				}
			} else {
				log::error!("Missing classname for node.");
			}
		}
	}
	fn get_all_publicpowergrid(&self, _pattern:&str) -> & Option<node::PublicPowerGrid> {
		& self.publicpowergrid
	}
	fn get_all_switch(&self, _pattern:&str) -> &Vec<node::Switch> {
		& self.switch
	}
}

#[derive(fmt::Debug)]
pub struct Network<Updater:HomeStateUpdater> {
    network_updater: Updater,
    nodes: NodesHeap,
    margin_power_on: f32,
	margin_power_on_loop_nb: u32,
	server: u64
}

impl<'a, Updater:HomeStateUpdater+fmt::Display> fmt::Display for Network<Updater> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{}> (\n", self.network_updater)?;
		for node in self.nodes.get_all("all") {
			write!(f, " - {}\n", node)?;
		}
		write!(f, ")")
    }
}

pub fn get<'a, Updater:HomeStateUpdater+fmt::Display>(network_updater:Updater) -> Network<Updater> {
	let network = Network {
		network_updater: network_updater,
		nodes: NodesHeap {
			publicpowergrid: None,
			switch: Vec::new(),
		},
		margin_power_on: -1.0,
		margin_power_on_loop_nb: 0,
		server: 0
	};
	// println!("INFO {network}");
	network
}

impl<Updater:HomeStateUpdater> Network<Updater> {
	pub fn set_nodes(&mut self, nodes_conf:Vec<&Yaml>) {
		self.nodes.set_nodes(&mut self.network_updater, nodes_conf);
	}
}