use std::{cell::RefCell, cmp::min, fmt::Debug, rc::Rc, thread::sleep, time::Duration};
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
	loopdelay: i64,
	strategies: Vec<OffPeakStrategy>,
	cycleid: u32,
	_allowsleep: bool,
	now: LocalDateTime,
	_inoverloadmode: bool,
	_errors: Vec<String>
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
		let loopdelay = configurator.get_as_int("server.loopDelay") as i64;
		assert!(loopdelay>=0);
		let strategies = Vec::new();
		let hems_server = Server{
			network: Rc::new(RefCell::new(Network::new(configurator)?)),
			loopdelay: loopdelay,
			strategies: strategies,
			cycleid: 0,
			_allowsleep: allowsleep,
			now: now,
			_inoverloadmode: false,
			_errors: Vec::new()
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
						let mut id: &String = &default;
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
		log::info!("Now: {:?}", now);
		self.now = now;
		let mut sleep_duration = self.loopdelay;
		{
			let mut network = self.network.borrow_mut();
			if let Err(err) = network.update() {
				log::error!("Fail update network : {}", err.message);
				return;
			}
		}
		for strategy in self.strategies.iter_mut() {
			match strategy.update_network(now) {
				Ok(time2sleep) => {
					sleep_duration = min(sleep_duration, time2sleep);
				}
				Err(err) => {
					log::error!("Fail update strategy {} : {err}", strategy.get_id());
				}
			}
		}
	}
	pub fn run(&mut self) {
		log::info!("Run OpenHEMS core server with loop-delay={}", self.loopdelay);
		loop {
			let now = LocalDateTime::now();
			let nextloop = now.add_seconds(self.loopdelay);
			self.loop1(now);
			let t = LocalDateTime::now();
			if t<nextloop {
				let secs = (nextloop.to_instant().seconds() - t.to_instant().seconds())  as u64;
				log::info!("Sleep for {} seconds.", secs);
				sleep(Duration::from_secs(secs));
			} else if t>nextloop {
				let secs = (t.to_instant().seconds() - nextloop.to_instant().seconds())  as u64;
				log::warn!("Missing {secs} seconds for the loop.");
			}
		}
	}

}