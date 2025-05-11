use core::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use arrayvec::ArrayString;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::feeder::{Feeder, SourceFeeder};
use crate::contract::Contract;
use crate::network::Network;

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
	fn get_current_power(&mut self) -> ResultOpenHems<f32>;
	fn is_on(&mut self) -> ResultOpenHems<bool>;
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

#[derive(Clone)]
pub struct NodeBase<'a, 'b:'a> {
	nameid: ArrayString<16>,
	max_power: f32,
	min_power: f32,
	current_power: SourceFeeder<'a, f32>,
	is_activate: bool,
	is_on: Feeder<'a, bool>,
	network: &'b Network<'a>
}
impl<'a, 'b:'a, 'c:'b> Debug for NodeBase<'a, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "id:{}, maxPower:{}, , minPower:{}, activ:{}, on:",
			self.nameid, self.max_power, self.min_power, self.is_activate, 
		)
    }
}

pub fn get_nodebase<'a>(network:&'a Network, nameid: &str, max_power: f32, min_power: f32, current_power:SourceFeeder<'a, f32>, is_on:Feeder<'a, bool>)
		-> ResultOpenHems<NodeBase<'a, 'a>> {
	if let Ok(name) = ArrayString::from(nameid) {
		Ok(NodeBase {
			nameid: name,
			max_power: max_power,
			min_power: min_power,
			current_power: current_power,
			is_activate: true,
			is_on: is_on,
			network: network
		})
	} else {
		Err(OpenHemsError::new(format!("'id' is to long (Limit is 16) for node {nameid}.")))
	}
}
impl<'a, 'b:'a, 'c:'b> Node for NodeBase<'a, 'a> {
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
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		Ok(self.current_power.get_value()?)
	}
    fn is_on(&mut self) -> ResultOpenHems<bool> {
		Ok(self.is_on.get_value()?)
	}
    fn is_activate(&mut self) -> bool {
		self.is_activate
	}
	fn get_type(&self) -> NodeType {
		NodeType::NodeBase
	}
}

#[derive(Clone, Debug)]
pub struct Switch<'a> {
	// Node
	node: NodeBase<'a, 'a>,
	// Outnode
	// Switch
	pritority: u32,
	strategy_nameid: ArrayString<16>,
	schedule: u32
}
pub fn get_switch<'a, 'b:'a, 'c:'b>(node: NodeBase<'a, 'a>, pritority: u32, strategy_nameid: &str) -> ResultOpenHems<Switch<'a>> {
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
impl<'a, 'b:'a, 'c:'b> Deref for Switch<'a> {
    type Target = NodeBase<'a, 'a>;
    fn deref(&self) -> &NodeBase<'a, 'a> {
        &self.node
    }
}
impl<'a, 'b:'a, 'c:'b> Node for Switch<'a> {
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
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		self.node.get_current_power()
	}
    fn is_on(&mut self) -> ResultOpenHems<bool> {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::Switch
	}
}

#[derive(Clone, Debug)] // Clone, 
pub struct PublicPowerGrid<'a> {
	// Node
	node: NodeBase<'a, 'a>,
	// Outnode
	// PublicPowerGrid
	contract: Contract
}
impl<'a, 'b:'a, 'c:'b> PublicPowerGrid<'a> {
	pub fn get_contract(&self) -> &Contract {
		&self.contract
	}
}
pub fn get_publicpowergrid<'a, 'b:'a, 'c:'b>(node: NodeBase<'a, 'a>, contract: Contract) -> ResultOpenHems<PublicPowerGrid<'a>> {
	Ok(PublicPowerGrid {
		node: node,
		contract: contract
	})
}
impl<'a, 'b:'a, 'c:'b> Deref for PublicPowerGrid<'a> {
    type Target = NodeBase<'a, 'a>;
    fn deref(&self) -> &NodeBase<'a, 'a> {
        &self.node
    }
}
impl<'a, 'b:'a, 'c:'b> Node for PublicPowerGrid<'a> {
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
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		self.node.get_current_power()
	}
    fn is_on(&mut self) -> ResultOpenHems<bool> {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::PublicPowerGrid
	}
}
