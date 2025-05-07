use std::collections::HashMap;
use yaml_rust2::Yaml;
use std::fmt;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::node::{self, PublicPowerGrid};
use crate::home_assistant_api::HomeStateUpdater;
use crate::cast_utility;
use crate::time::HoursRanges;

// Rust equivalent of Python nodes list with multi nodes types.
#[derive(Clone)]
pub struct NodesHeap<'a, Updater:HomeStateUpdater+Clone> {
	publicpowergrid: Option<node::PublicPowerGrid<'a, Updater>>,
	switch: Vec<node::Switch<'a, Updater>>,
	// solarpanel: Vec<node::SolarPanel>,
	// battery: Vec<node::Battery>,
}
struct NodesHeapIterator<'b, Updater:HomeStateUpdater+Clone+'b> {
	nodetype: node::NodeType,
	index:usize,
	filter: String,
	heap: &'b NodesHeap<'b, Updater>
}
/*
impl<'a, Updater:HomeStateUpdater+Clone> fmt::Debug for NodesHeap<'a, Updater> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		let iter: NodesHeapIterator<'_, Updater> = self.get_all();
		for node in iter {
			let _ = write!(f, ", {}", node);
		}
		Ok(())
    }
} */
impl<'a, Updater:HomeStateUpdater+Clone> Iterator for NodesHeapIterator<'a, Updater> {
    // We can refer to this type using Self::Item
    type Item = Box<&'a dyn node::Node>;

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

impl<'a, Updater:HomeStateUpdater+Clone> NodesHeap<'a, Updater> {
	pub fn get_all<'b>(&'static self) -> NodesHeapIterator<'b, Updater>
	{
		NodesHeapIterator {
			nodetype: node::NodeType::PublicPowerGrid,
			index: 0,
			filter: "all".to_string(),
			heap: &self,
		}
	}
	pub fn set_nodes(&mut self, updater:&'a mut Updater, nodes_conf:Vec<&Yaml>) {
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
						match updater.get_switch(nameid.as_str(), &node_conf) {
							Ok(node) => {
								self.switch.push(node);
							}
							Err(err) => {
								let message = format!("Impossible to add switch '{nameid}' due to {}.", err.message);
								log::error!("ERROR {}",&message);
							}
						}
					},
					"publicpowergrid" => {
						match updater.get_publicpowergrid(nameid.as_str(), &node_conf) {
							Ok(node) => {
								self.publicpowergrid = Some(node);
							}
							Err(err) => {
								let message = format!("Impossible to add PublicPowerGrid '{nameid}' due to {}.", err.message);
								log::error!("ERROR {}",&message);
							}
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
	}
	fn get_publicpowergrid(&self) -> & Option<node::PublicPowerGrid<'a, Updater>> {
		& self.publicpowergrid
	}
	fn get_all_switch(&self, _pattern:&str) -> &Vec<node::Switch<'a, Updater>> {
		& self.switch
	}
}

#[derive(Clone)]
pub struct Network<'a, Updater:HomeStateUpdater+Clone> {
    pub updater: Updater,
    nodes: NodesHeap<'a, Updater>,
    margin_power_on: f32,
	margin_power_on_loop_nb: u32,
	server: u64
}
/*
impl<'a, Updater:HomeStateUpdater+fmt::Display+Clone> fmt::Display for Network<'a, Updater> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Network<{}> (\n", self.updater)?;
		let iter = self.nodes.get_all();
		for node in  iter {
			write!(f, " - {}\n", node)?;
		}
		write!(f, ")")
    }
} */

impl<'a, Updater> Network<'a, Updater> 
	where Updater:HomeStateUpdater+Clone
{
	pub fn default() -> Network<'a, Updater> {
		Network {
			updater: Updater::default(),
			nodes: NodesHeap {
				publicpowergrid: None,
				switch: Vec::new(),
			},
			margin_power_on: -1.0,
			margin_power_on_loop_nb: 0,
			server: 0
		}
	}
	pub fn set_nodes(&'a mut self, nodes_conf:Vec<&Yaml>) {
		self.nodes.set_nodes(&mut self.updater, nodes_conf);
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
