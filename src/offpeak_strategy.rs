use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use datetime::LocalDateTime;
use hashlink::linked_hash_map::LinkedHashMap;
use crate::error::{OpenHemsError, ResultOpenHems};
use crate::network::{self, Network};
use crate::node::Node;
use crate::time::{HoursRanges, HoursRange};
use yaml_rust2::Yaml;

pub trait EnergyStrategy {
	fn get_strategy_id(&self) -> &str;
	fn get_nodes(&self) -> &Vec<Box<dyn Node>>;
	fn update_network(&self) -> ResultOpenHems<u32>;
	fn new(network:Rc<RefCell<Network>>, id:&str, config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy>;
}
// #[derive(Clone)]
pub struct OffPeakStrategy {
	id: String,
	inoffpeakrange: bool,
	rangechangedone: bool,
	nextranges: Vec<HoursRange>,
	network: Rc<RefCell<Network>>,
}

impl<'a, 'b:'a> EnergyStrategy for OffPeakStrategy {
	fn get_strategy_id(&self) -> &str {
		&self.id
	}
	fn get_nodes(&self) -> &Vec<Box<dyn Node>> {
		todo!();
	}
	fn update_network(&self) -> ResultOpenHems<u32>{
		todo!();
		// Ok(0)
	}
	fn new(network:Rc<RefCell<Network>>, id:&str, _config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy> {
		Ok(OffPeakStrategy {
			id: id.to_string(),
			inoffpeakrange: false,
			rangechangedone: false,
			nextranges: Vec::new(),
			network: network
		})
	}
}

impl<'a, 'b:'a, 'c:'b, 'd:'c> OffPeakStrategy {
	pub fn get_id(&self) -> &str {
		&self.id
	}
	fn init(& mut self, now:LocalDateTime) -> ResultOpenHems<()> {
		/* .map_err(
			|message| OpenHemsError::new(format!("Fail lock network : {}", message.to_string()))
		)?; */
		let network = self.network.borrow_mut();
		let hoursranges = network.get_hours_ranges()?;
		let range = hoursranges.check_range(now)?;
		self.inoffpeakrange = hoursranges.is_offpeak(range);
		Ok(())
	}
}
