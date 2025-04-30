use core::fmt;
use std::ops::Deref;
use std::marker::Copy;
use arrayvec::ArrayString;
use std::collections::HashMap;
use yaml_rust2::Yaml;
use log;
use crate::home_assistant_api::HomeStateUpdater;
use crate::error::{OpenHemsError, ResultOpenHems};

pub trait NodeTrait {
	fn get_type(&self) -> NodeType;
	fn get_id(&self) -> &str;
    fn get_min_power(&self) -> f32;
    fn get_max_power(&self) -> f32;
    fn get_current_power(&self) -> f32;
    fn is_on(&self) -> bool;
    fn is_activate(&self) -> bool;
}
pub enum NodeType {
    // #[lang = "NodeBase"]
	NodeBase,
    // #[lang = "Switch"]
    Switch,
	PublicPowerGrid,
}
pub union NodeUnion {
	base:NodeBase,
    switch:Switch,
	powergrid:PublicPowerGrid,
}

pub struct Node {
	tag: NodeType,
	val: NodeUnion,
}
pub fn get_node<Updater:HomeStateUpdater>(updater:&mut Updater, classname:&str, nameid: String, node_conf: HashMap<String, &Yaml>) -> ResultOpenHems<Node> {
	match &*classname.to_lowercase() {
		"switch" => {
			let node = updater.get_switch(&nameid, node_conf)?;
			let wrapping_node = Node {
				tag: NodeType::Switch,
				val: NodeUnion{switch: node}
			};
			Ok(wrapping_node)
		},
		"publicpowergrid" => {
			let node = updater.get_publicpowergrid(&nameid, node_conf)?;
			let wrapping_node = Node {
				tag: NodeType::PublicPowerGrid,
				val: NodeUnion{powergrid: node}
			};
			Ok(wrapping_node)
		},
		_ => {
			let message = format!("Unknwon class '{classname}'");
			log::error!("ERROR {}",&message);
			Err(OpenHemsError::new(message))
		}
	}
}
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Node(, MaxPower)")
    }
}
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Node(, MaxPower:)")
    }
}
impl Node {
	fn to_type_nodebase(&self) -> Option<NodeBase> {
		unsafe {
			match self.tag {
				NodeType::NodeBase => {
					Some(self.val.base)
				}
				_ => {
					None
				}
			}
		}
	}
	fn to_type_publicpowergrid(&self) -> Option<PublicPowerGrid> {
		unsafe {
			match self.tag {
				NodeType::PublicPowerGrid => {
					Some(self.val.powergrid)
				}
				_ => {
					None
				}
			}
		}
	}
	fn to_type_switch(&self) -> Option<Switch> {
		unsafe {
			match self.tag {
				NodeType::PublicPowerGrid => {
					Some(self.val.switch)
				}
				_ => {
					None
				}
			}
		}
	}
}

#[derive(Copy, Clone)]
pub struct NodeBase {
	nameid: ArrayString<16>,
	max_power: f32,
	min_power: f32,
	current_power: f32,
	is_activate: bool,
	is_on: bool
}

pub fn get_nodebase(nameid: &str, max_power: f32, min_power: f32, current_power:f32, is_on:bool) -> ResultOpenHems<NodeBase> {
	if let Ok(name) = ArrayString::from(nameid) {
		Ok(NodeBase {
			nameid: name,
			max_power: max_power,
			min_power: min_power,
			current_power: current_power,
			is_activate: true,
			is_on: is_on
		})
	} else {
		Err(OpenHemsError::new(format!("'id' is to long (Limit is 16) for node {nameid}.")))
	}
}
impl NodeTrait for NodeBase {
    // Attributes
	fn get_id(&self) -> &str {
		&self.nameid
	}
    fn get_min_power(&self) -> f32 {
		self.min_power
	}
    fn get_max_power(&self) -> f32 {
		self.max_power
	}
    fn get_current_power(&self) -> f32 {
		self.current_power
	}
    fn is_on(&self) -> bool {
		self.is_on
	}
    fn is_activate(&self) -> bool {
		self.is_activate
	}
	fn get_type(&self) -> NodeType {
		NodeType::NodeBase
	}
}

#[derive(Copy, Clone)]
pub struct Switch {
	// Node
	node: NodeBase,
	// Outnode
	// Switch
	pritority: u32,
	strategy_nameid: ArrayString<16>,
	schedule: u32
}
pub fn get_switch(node: NodeBase, pritority: u32, strategy_nameid: &str) -> ResultOpenHems<Switch> {
	if let Ok(strategy) = ArrayString::from(strategy_nameid) {
		Ok(Switch {
			node: node,
			pritority: pritority,
			strategy_nameid: strategy,
			schedule: 0
		})
	} else {
		Err(OpenHemsError::new("Strategy is to long (Limit is 16)".to_string()))
	}
}
impl Deref for Switch {
    type Target = NodeBase;
    fn deref(&self) -> &NodeBase {
        &self.node
    }
}

#[derive(Copy, Clone)]
pub struct PublicPowerGrid {
	// Node
	node: NodeBase,
	// Outnode
	// PublicPowerGrid
	contract: u32
}
pub fn get_publicpowergrid(node: NodeBase, contract: u32) -> ResultOpenHems<PublicPowerGrid> {
	Ok(PublicPowerGrid {
		node: node,
		contract: contract
	})
}
impl Deref for PublicPowerGrid {
    type Target = NodeBase;
    fn deref(&self) -> &NodeBase {
        &self.node
    }
}
