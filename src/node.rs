use core::fmt;
use std::ops::Deref;
use arrayvec::ArrayString;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::feeder::{Feeder, SourceFeeder};
use crate::home_assistant_api::HomeStateUpdater;

#[derive(Clone)]
pub enum NodeType {
    // #[lang = "NodeBase"]
	NodeBase,
    // #[lang = "Switch"]
    Switch,
	PublicPowerGrid,
}
impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s:&str;
		match self {
			NodeType::NodeBase => {
				s = "NodeBase";
			}
			NodeType::Switch => {
				s = "Switch";
			}
			NodeType::PublicPowerGrid => {
				s = "PublicPowerGrid";
			}
		}
        write!(f, "{}", s)
    }
}
impl fmt::Debug for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s:&str;
		match self {
			NodeType::NodeBase => {
				s = "NodeBase";
			}
			NodeType::Switch => {
				s = "Switch";
			}
			NodeType::PublicPowerGrid => {
				s = "PublicPowerGrid";
			}
		}
        write!(f, "{}", s)
    }
}
pub trait Node {
	fn get_type(&self) -> NodeType;
	fn get_id(&self) -> &str;
	fn get_min_power(&mut self) -> f32;
	fn get_max_power(&mut self) -> f32;
	fn get_current_power(&mut self) -> f32;
	fn is_on(&mut self) -> bool;
	fn is_activate(&mut self) -> bool;
}
impl fmt::Display for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}({})", self.get_type(), self.get_id())
    }
}
impl fmt::Debug for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}({})", self.get_type(), self.get_id())
    }
}

#[derive(Clone, Debug)]
pub struct NodeBase<'a, Updater:HomeStateUpdater+Clone> {
	nameid: ArrayString<16>,
	max_power: f32,
	min_power: f32,
	current_power: SourceFeeder<'a, Updater, f32>,
	is_activate: bool,
	is_on: bool
}

pub fn get_nodebase<'a, Updater:HomeStateUpdater+Clone>(nameid: &str, max_power: f32, min_power: f32, current_power:SourceFeeder<'a, Updater, f32>, is_on:bool) -> ResultOpenHems<NodeBase<'a, Updater>> {
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
impl<'a, Updater:HomeStateUpdater+Clone> Node for NodeBase<'a, Updater> {
    // Attributes
	fn get_id(&self) -> &str {
		&self.nameid
	}
    fn get_min_power(&mut self) -> f32 {
		self.min_power
	}
    fn get_max_power(&mut self) -> f32 {
		self.max_power
	}
    fn get_current_power(&mut self) -> f32 {
		self.current_power.get_value().unwrap()
	}
    fn is_on(&mut self) -> bool {
		self.is_on
	}
    fn is_activate(&mut self) -> bool {
		self.is_activate
	}
	fn get_type(&self) -> NodeType {
		NodeType::NodeBase
	}
}

#[derive(Clone, Debug)]
pub struct Switch<'a, Updater:HomeStateUpdater+Clone> {
	// Node
	node: NodeBase<'a, Updater>,
	// Outnode
	// Switch
	pritority: u32,
	strategy_nameid: ArrayString<16>,
	schedule: u32
}
pub fn get_switch<'a, Updater:HomeStateUpdater+Clone>(node: NodeBase<'a, Updater>, pritority: u32, strategy_nameid: &str) -> ResultOpenHems<Switch<'a, Updater>> {
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
impl<'a, Updater:HomeStateUpdater+Clone> Deref for Switch<'a, Updater> {
    type Target = NodeBase<'a, Updater>;
    fn deref(&self) -> &NodeBase<'a, Updater> {
        &self.node
    }
}
impl<'a, Updater:HomeStateUpdater+Clone> Node for Switch<'a, Updater> {
    // Attributes
	fn get_id(&self) -> &str {
		self.node.get_id()
	}
    fn get_min_power(&mut self) -> f32 {
		self.node.get_min_power()
	}
    fn get_max_power(&mut self) -> f32 {
		self.node.get_max_power()
	}
    fn get_current_power(&mut self) -> f32 {
		self.node.get_current_power()
	}
    fn is_on(&mut self) -> bool {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::Switch
	}
}

#[derive(Clone, Debug)]
pub struct PublicPowerGrid<'a, Updater:HomeStateUpdater+Clone> {
	// Node
	node: NodeBase<'a, Updater>,
	// Outnode
	// PublicPowerGrid
	contract: u32
}
pub fn get_publicpowergrid<Updater:HomeStateUpdater+Clone>(node: NodeBase<Updater>, contract: u32) -> ResultOpenHems<PublicPowerGrid<Updater>> {
	Ok(PublicPowerGrid {
		node: node,
		contract: contract
	})
}
impl<'a, Updater:HomeStateUpdater+Clone> Deref for PublicPowerGrid<'a, Updater> {
    type Target = NodeBase<'a, Updater>;
    fn deref(&self) -> &NodeBase<'a, Updater> {
        &self.node
    }
}
impl<'a, Updater:HomeStateUpdater+Clone> Node for PublicPowerGrid<'a, Updater> {
    // Attributes
	fn get_id(&self) -> &str {
		self.node.get_id()
	}
    fn get_min_power(&mut self) -> f32 {
		self.node.get_min_power()
	}
    fn get_max_power(&mut self) -> f32 {
		self.node.get_max_power()
	}
    fn get_current_power(&mut self) -> f32 {
		self.node.get_current_power()
	}
    fn is_on(&mut self) -> bool {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::PublicPowerGrid
	}
}
