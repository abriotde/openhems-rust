use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use yaml_rust2::Yaml;
use std::fmt::{self, Display};
use crate::configuration_manager::ConfigurationManager;
use crate::contract::Contract;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::node::{self, Node};
use crate::home_assistant_api::{HomeStateUpdater,HomeAssistantAPI};
use crate::cast_utility;
use crate::time::HoursRanges;
use crate::web::AppState;

// Rust equivalent of Python nodes list with multi nodes types.
#[derive(Clone, Debug)]
pub struct NodesHeap {
	publicpowergrid: Option<node::PublicPowerGrid>,
	switch: Vec<node::Switch>,
	solarpanel: Vec<node::SolarPanel>,
	// battery: Vec<node::Battery>,
}
#[derive(Clone, Debug)]
pub struct NodesHeapIterator<'a> {
	nodetype: node::NodeType,
	index:usize,
	_filter: String,
	heap: &'a NodesHeap
}
/* impl fmt::Debug for NodesHeap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		for node in self.get_all() {
			let _ = write!(f, ", {}", node);
		}
		Ok(())
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

impl<'a, 'b:'a> NodesHeap {
	pub fn new() -> NodesHeap {
		NodesHeap {
			publicpowergrid: None,
			switch: Vec::new(),
			solarpanel: Vec::new(),
			// battery: Vec<node::Battery>,
		}
	}
	pub fn get_all(&'b self) -> NodesHeapIterator<'b>
	{
		NodesHeapIterator {
			nodetype: node::NodeType::PublicPowerGrid,
			index: 0,
			_filter: "all".to_string(),
			heap: self,
		}
	}
	pub fn set_switch(& mut self, nameid:&str, updater:Rc<RefCell<HomeAssistantAPI>>, 
				node_conf:&HashMap<String, &Yaml>, appstate:&mut AppState
			) -> ResultOpenHems<()> {
		// println!("set_switch({nameid})");
		let priority = HomeAssistantAPI::get_feeder_const_int(node_conf, "priority", 50);
		let strategy_nameid = HomeAssistantAPI::get_feeder_const_str(node_conf, "strategy", "default");
		let base = HomeAssistantAPI::get_nodebase(updater, nameid, node_conf)?;
		let switch = node::get_switch(base, priority as u32, &strategy_nameid, appstate)?;
		self.switch.push(switch);
		log::debug!("set_switch({nameid}) : Ok");
		Ok(())
	}
	pub fn set_publicpowergrid(& mut self, nameid:&str, updater:Rc<RefCell<HomeAssistantAPI>>, node_conf:&HashMap<String, &Yaml>)  -> ResultOpenHems<()> {
		// println!("set_publicpowergrid()");
		let base = HomeAssistantAPI::get_nodebase(updater, nameid, node_conf)?;
		if let Some(contract_conf) = node_conf.get("contract") {
			let contract = Contract::get_from_conf(contract_conf)?;
			let node = node::get_publicpowergrid(base, contract)?;
			log::debug!("set_publicpowergrid({nameid}) : Ok");
			self.publicpowergrid = Some(node);
			Ok(())
		} else {
			Err(OpenHemsError::new(format!(
				"No key 'contract' in publicpowergrid configuration."
			)))
		}
	}
	pub fn get_publicpowergrid(&self) -> & Option<node::PublicPowerGrid> {
		& self.publicpowergrid
	}
	pub fn get_all_switch(&self, _pattern:&str) -> &Vec<node::Switch> {
		& self.switch
	}
	pub fn get_all_switch_mut(&mut self, _pattern:&str) -> &mut Vec<node::Switch> {
		&mut self.switch
	}
	pub fn get_all_solarpanel(&self, _pattern:&str) -> &Vec<node::SolarPanel> {
		& self.solarpanel
	}
	pub fn get_current_power(&self, filter:&str) -> ResultOpenHems<f32> {
		let mut current_power = 0.0;
		if ["", "all", "publicpowergrid"].iter().any(|&s| s==filter) {
			if let Some(power) = &self.publicpowergrid {
				current_power += power.clone().get_current_power()?;
			}
		}
		if ["", "all", "solarpanel"].iter().any(|&s| s==filter) {
			for solarpanel in &self.solarpanel {
				current_power += solarpanel.clone().get_current_power()?;
			}
		}
		Ok(current_power)
	}
}

#[derive(Clone, Debug)]
pub struct Network {
    updater: Rc<RefCell<HomeAssistantAPI>>,
    nodes: NodesHeap,
    _margin_power_on: f32,
	_margin_power_on_cache_id: u32,
	errors: Vec<String>,
}
impl<'a, 'b:'a> Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{:?}> (\n{:?}\n)", self.updater, self.nodes)
    }
}
impl Deref for Network 
{
	type Target = NodesHeap;
	fn deref(&self) -> &<Self as Deref>::Target {
		&self.nodes
	}
}

impl Network
{
	pub fn new(configurator:&ConfigurationManager) -> ResultOpenHems<Network> {
		let margin_power_on = 0.0;
		let margin_power_on_cache_id = 0;
		let updater:HomeAssistantAPI;
		let network_source = configurator.get_as_str("server.network");
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
			updater: Rc::new(RefCell::new(updater)),
			nodes: NodesHeap::new(),
			_margin_power_on: margin_power_on,
			_margin_power_on_cache_id: margin_power_on_cache_id,
			errors: Vec::new()
		};
		Ok(network)
	}
	pub fn set_nodes(&mut self, configurator:&ConfigurationManager, appstate:&mut AppState) -> () {
		let nodes_conf = configurator.get_as_list("network.nodes");
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
				match &*classname.to_lowercase() {
					"switch" => {
						if let Err(err) = self.nodes.set_switch(nameid.as_str(), self.updater.clone(), &node_conf, appstate) {
							let message = format!("Impossible to add switch '{nameid}' due to {}.", err.message);
							log::error!("ERROR {}",&message);
							self.errors.push(message);
						}
					},
					"publicpowergrid" => {
						if let Err(err) = self.nodes.set_publicpowergrid(nameid.as_str(), self.updater.clone(), &node_conf) {
							let message = format!("Impossible to add PublicPowerGrid '{nameid}' due to {}.", err.message);
							log::error!("ERROR {}",&message);
							self.errors.push(message);
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
		println!("Nodes:{:?}", self.nodes);
	}
	pub fn get_hours_ranges(&self) -> ResultOpenHems<&HoursRanges> {
		if let Some(power) = self.nodes.get_publicpowergrid() {
			let hoursranges = power.get_contract().get_hoursranges();
			Ok(hoursranges)
		} else {
			Err(OpenHemsError::new("Need a public power grid for hours ranges but there is not.".to_string()))
		}
	}
	pub fn update(&mut self) -> ResultOpenHems<bool> {
		let mut updater = self.updater.borrow_mut();
		updater.update_network()
	}
	pub fn notify(&self, message:&str) -> ResultOpenHems<bool> {
		self.updater.borrow().notify(message)
	}
}
