use core::net;
use std::{cell::RefCell, cmp::min, fmt::Debug, rc::Rc, sync::{Arc, Mutex}};
use datetime::LocalDateTime;
use yaml_rust2::Yaml;
use crate::{
	error::{OpenHemsError, ResultOpenHems},
	network::Network,
	offpeak_strategy::{EnergyStrategy, OffPeakStrategy},
	utils::get_yaml_key,
	configuration_manager::ConfigurationManager
};

// #[derive(Clone)]
pub struct Server {
	pub network: Rc<RefCell<Network>>,
	loopdelay: u32,
	strategies: Vec<OffPeakStrategy>,
	cycleid: u32,
	allowsleep: bool,
	now: LocalDateTime,
	inoverloadmode: bool,
	errors: Vec<String>
}
impl<'a> Debug for Server {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "Server(loopdelay:{}, cycleid:{}) On \n{:?}\n", self.loopdelay, self.cycleid, self.network)
	}
}
impl<'a> Server {
	pub fn new(configurator: &ConfigurationManager) -> ResultOpenHems<Server> {
		let allowsleep = true;
		let now: LocalDateTime = LocalDateTime::at(0);
		let loopdelay = configurator.get_as_int("server.loopdelay") as u32;
		let strategies = Vec::new();
		let hems_server = Server{
			network: Rc::new(RefCell::new(Network::new(configurator)?)),
			loopdelay: loopdelay,
			strategies: strategies,
			cycleid: 0,
			allowsleep: allowsleep,
			now: now,
			inoverloadmode: false,
			errors: Vec::new()
		};
		Ok(hems_server)
	}
	pub fn init(&mut self, configurator: &ConfigurationManager) -> ResultOpenHems<()> {
		let mut network = self.network.borrow_mut();
		network.set_nodes(configurator);
		if let Some(configuration) = configurator.get("server.strategies") {
			if let Some(list) = configuration.clone().into_vec() {
				let default = String::from("");
				for config in list {
					if let Yaml::Hash(conf) = &config {
						let mut classname = &default;
						if let Some(Yaml::String(v)) = get_yaml_key("class", conf) {
							classname = v;
						} else {
							return Err(OpenHemsError::new(format!(
								"Missing key 'id' for strategy."
							)));
						}
						let mut id = &default;
						if let Some(Yaml::String(v)) = get_yaml_key("id", conf) {
							id = v;
						} else {
							return Err(OpenHemsError::new(format!(
								"Missing key 'id' for strategy."
							)));
						}
						match classname.to_lowercase().as_str() {
							"offpeak" => {
								self.strategies.push(OffPeakStrategy::new(Rc::clone(&self.network), &id, conf)?);
							}
							_ => {
								return Err(OpenHemsError::new(format!(
									"Not supported strategy class : '{classname}'."
								)));
							}
							
						}
					}
				}
			}
		}
		Ok(())
	}
	pub fn loop1(&mut self, now:LocalDateTime) {
		self.now = now;
		
		/* if let Err(err) = self.network.update() {
			log::error!("Fail update network");
		} */
		let mut sleep_duration = self.loopdelay;
		for strategy in self.strategies.iter() {
			match strategy.update_network() {
				Ok(time2sleep) => {
					sleep_duration = min(sleep_duration, time2sleep);
				}
				Err(err) => {
					log::error!("Fail update strategy {} : {err}", strategy.get_id());
				}
			}
		}
	}
	pub fn run(&self, loopdelay:u32) {
		todo!()
	}

}