use core::net;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Local;
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
	fn update_network(&mut self, now:LocalDateTime) -> ResultOpenHems<i64>;
	fn new(network:Rc<RefCell<Network>>, id:&str, config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy>;
}
// #[derive(Clone)]
pub struct OffPeakStrategy {
	id: String,
	inoffpeakrange: bool,
	rangechangedone: bool,
	nextranges: Vec<HoursRange>,
	network: Rc<RefCell<Network>>,
	rangeend: LocalDateTime
}

impl<'a, 'b:'a> EnergyStrategy for OffPeakStrategy {
	fn get_strategy_id(&self) -> &str {
		&self.id
	}
	fn get_nodes(&self) -> &Vec<Box<dyn Node>> {
		todo!();
	}
	fn update_network(&mut self, now:LocalDateTime) -> ResultOpenHems<i64> {
		if now>self.rangeend {
			let network = self.network.borrow_mut();
			let hoursranges = network.get_hours_ranges()?;
			let range = hoursranges.check_range(now)?;
			self.rangeend = range.get_end(now);
			self.inoffpeakrange = hoursranges.is_offpeak(range);
		}
		if self.inoffpeakrange {
			self.switch_on_max();
			self.rangechangedone = false;
		} else {
			if !self.rangechangedone {
				if self.switch_off_all() {
					self.rangechangedone = true;
				}
			}
		}
		Ok(100000)
	}
	fn new(network:Rc<RefCell<Network>>, id:&str, _config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy> {
		Ok(OffPeakStrategy {
			id: id.to_string(),
			inoffpeakrange: false,
			rangechangedone: false,
			rangeend: LocalDateTime::at(0),
			nextranges: Vec::new(),
			network: network
		})
	}
}

impl<'a, 'b:'a, 'c:'b, 'd:'c> OffPeakStrategy {
	pub fn get_id(&self) -> &str {
		&self.id
	}
	fn switch_on_max(&self) {
		log::debug!("OffPeakStrategy::switch_on_max()");
	}
	fn switch_off_all(&self) -> bool {
		log::debug!("OffPeakStrategy::switch_off_all()");
		let mut ok = true;
		let network = self.network.borrow_mut();
		for elem in network.get_all_switch("all") {
			if let Err(err) = network.switch(elem, false) {
				log::warn!("Fail switch off '{}' : {}", elem.get_id(), err.message);
				ok = false;
			}
		}
		ok
	}

}
