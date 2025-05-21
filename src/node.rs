use core::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, MutexGuard, Mutex};
use arrayvec::ArrayString;
use chrono::{DateTime, Local};
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::feeder::{Feeder, SourceFeeder};
use crate::contract::Contract;
use crate::schedule::Schedule;
use crate::time;
use crate::web::AppState;

#[derive(Clone)]
pub enum NodeType {
    // #[lang = "NodeBase"]
	NodeBase,
    // #[lang = "Switch"]
    Switch,
	PublicPowerGrid,
	SolarPanel,
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
			NodeType::SolarPanel => {
				s = "SolarPanel";
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
			NodeType::SolarPanel => {
				s = "SolarPanel";
			}
		}
        write!(f, "{}", s)
    }
}
impl NodeType {
}
pub trait Node {
	fn get_type(&self) -> NodeType;
	fn get_id(&self) -> &str;
	fn get_min_power(&self) -> f32;
	fn get_max_power(&self) -> f32;
	fn get_current_power(&mut self) -> ResultOpenHems<f32>;
	fn is_on(&self) -> ResultOpenHems<bool>;
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
pub struct NodeBase {
	nameid: ArrayString<16>,
	max_power: f32,
	min_power: f32,
	current_power: SourceFeeder<f32>,
	is_activate: bool,
	pub is_on: Feeder<bool>
}
impl<'a, 'b:'a, 'c:'b> Debug for NodeBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "id:{}, maxPower:{}, , minPower:{}, activ:{}, on:",
			self.nameid, self.max_power, self.min_power, self.is_activate, 
		)
    }
}

pub fn get_nodebase(nameid: &str, max_power: f32, min_power: f32, current_power:SourceFeeder<f32>, is_on:Feeder<bool>)
		-> ResultOpenHems<NodeBase> {
	if let Ok(name) = ArrayString::from(nameid) {
		Ok(NodeBase {
			nameid: name,
			max_power: max_power,
			min_power: min_power,
			current_power: current_power,
			is_activate: true,
			is_on: is_on,
		})
	} else {
		Err(OpenHemsError::new(format!("'id' is to long (Limit is 16) for node {nameid}.")))
	}
}
impl<'a, 'b:'a, 'c:'b> Node for NodeBase {
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
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		Ok(self.current_power.get_value()?)
	}
    fn is_on(&self) -> ResultOpenHems<bool> {
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
pub struct Switch {
	// Node
	node: NodeBase,
	// Outnode
	// Switch
	_pritority: u32,
	_strategy_nameid: ArrayString<16>,
	schedule: Arc<Mutex<Schedule>>
}
pub fn get_switch<'a, 'b:'a, 'c:'b>(node: NodeBase, pritority: u32, strategy_nameid: &str,
			appstate:&mut AppState
		) -> ResultOpenHems<Switch> {
	if let Ok(strategy) = ArrayString::from(strategy_nameid) {
		let sch = Arc::new(Mutex::new(Schedule::new(&node.nameid)));
		appstate.schedules.insert(node.nameid.to_string(), sch.clone());
		Ok(Switch {
			node: node,
			_pritority: pritority,
			_strategy_nameid: strategy,
			schedule: sch
		})
	} else {
		Err(OpenHemsError::new("Strategy is to long (Limit is 16)".to_string()))
	}
}
impl Switch {
	pub fn switch(&self, on:bool) -> ResultOpenHems<bool> {
		log::debug!("{}.switch(on={on})", self.get_id());
		if let Feeder::Source(mut feeder) = self.is_on.clone() {
			let on2 = if self.get_schedule().is_scheduled() {on} // Switch on only if scheduled
				else {false}; // else don't
			log::debug!("Switch {}: is_on={} -> is_scheduled={}", self.get_id(), feeder.get_value()?, on2);
			if feeder.get_value()?!=on2 {
				return feeder.switch(feeder.get_nameid().as_str(), on2);
			}
		}
		Ok(true)
	}
	pub fn get_schedule<'a>(&'a self) -> MutexGuard<'a, Schedule, > {
		self.schedule.lock().unwrap()
	}
	pub fn set_schedule(& mut self, duration:u32, timeout:Option<DateTime<Local>>) {
		self.get_schedule().set_duration(duration);
		if let Some(t) = timeout {
			self.get_schedule().set_timeout(&t);
		} else {
			self.get_schedule().set_timeout(&time::MIN_DATETIME);
		}
	}
}
impl<'a, 'b:'a, 'c:'b> Deref for Switch {
    type Target = NodeBase;
    fn deref(&self) -> &NodeBase {
        &self.node
    }
}
impl<'a, 'b:'a, 'c:'b> Node for Switch {
    // Attributes
	fn get_id(&self) -> &str {
		self.node.get_id()
	}
    fn get_min_power(&self) -> f32 {
		self.node.get_min_power()
	}
    fn get_max_power(&self) -> f32 {
		self.node.get_max_power()
	}
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		self.node.get_current_power()
	}
    fn is_on(&self) -> ResultOpenHems<bool> {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::Switch
	}
}

#[derive(Clone, Debug)] // Clone
pub struct PublicPowerGrid {
	// Node
	node: NodeBase,
	// Outnode
	// PublicPowerGrid
	contract: Contract
}
impl<'a, 'b:'a, 'c:'b> PublicPowerGrid {
	pub fn get_contract(&self) -> &Contract {
		&self.contract
	}
}
pub fn get_publicpowergrid<'a, 'b:'a, 'c:'b>(node: NodeBase, contract: Contract) -> ResultOpenHems<PublicPowerGrid> {
	Ok(PublicPowerGrid {
		node: node,
		contract: contract
	})
}
impl<'a, 'b:'a, 'c:'b> Deref for PublicPowerGrid {
    type Target = NodeBase;
    fn deref(&self) -> &NodeBase {
        &self.node
    }
}
impl<'a, 'b:'a, 'c:'b> Node for PublicPowerGrid {
    // Attributes
	fn get_id(&self) -> &str {
		self.node.get_id()
	}
    fn get_min_power(&self) -> f32 {
		self.node.get_min_power()
	}
    fn get_max_power(&self) -> f32 {
		self.node.get_max_power()
	}
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		self.node.get_current_power()
	}
    fn is_on(&self) -> ResultOpenHems<bool> {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::PublicPowerGrid
	}
}

#[derive(Clone, Debug)] // Clone
pub struct SolarPanel {
	// Node
	node: NodeBase,
	module_model: String,
	inverter_model: String,
	tilt: f32,
	azimuth: f32,
	module_per_string: u32,
	strings_per_inverter: u32,
}
impl SolarPanel {
}
pub fn get_solarpanel(node: NodeBase, module_model: String, inverter_model: String,
			tilt: f32, azimuth: f32, module_per_string: u32, strings_per_inverter: u32
		) -> ResultOpenHems<SolarPanel> {
	Ok(SolarPanel {
		node: node,
		module_model,
		inverter_model,
		tilt,
		azimuth,
		module_per_string,
		strings_per_inverter,
	})
}
impl Deref for SolarPanel {
    type Target = NodeBase;
    fn deref(&self) -> &NodeBase {
        &self.node
    }
}
impl Node for SolarPanel {
    // Attributes
	fn get_id(&self) -> &str {
		self.node.get_id()
	}
    fn get_min_power(&self) -> f32 {
		self.node.get_min_power()
	}
    fn get_max_power(&self) -> f32 {
		self.node.get_max_power()
	}
    fn get_current_power(&mut self) -> ResultOpenHems<f32> {
		self.node.get_current_power()
	}
    fn is_on(&self) -> ResultOpenHems<bool> {
		self.node.is_on()
	}
    fn is_activate(&mut self) -> bool {
		self.node.is_activate()
	}
	fn get_type(&self) -> NodeType {
		NodeType::PublicPowerGrid
	}
}
