use core::net;
use std::cmp::min;
use datetime::LocalDateTime;
use yaml_rust2::Yaml;
use crate::{
	error::{OpenHemsError, ResultOpenHems},
	home_assistant_api::HomeStateUpdater,
	network::Network,
	offpeak_strategy::{EnergyStrategy, OffPeakStrategy},
	utils::get_yaml_key,
	configuration_manager::ConfigurationManager
};

#[derive(Clone)]
pub struct Server<'a, 'b:'a> {
	network:&'b Network<'a, 'a>,
	loopdelay: u32,
	strategies: Vec<OffPeakStrategy<'a, 'a>>,
	cycleid: u32,
	allowsleep: bool,
	now: LocalDateTime,
	inoverloadmode: bool
}
impl<'a, 'b:'a, 'c:'b, 'd:'c> Server<'a, 'a> {
	pub fn new(network:&'a Network<'a, 'a>, configurator: &ConfigurationManager) -> ResultOpenHems<Server<'a, 'a>> {
		let mut strategies= Vec::new();
		let mut loopdelay = configurator.get_as_int("server.loopdelay") as u32;
		let mut allowsleep = true;
		let mut now = LocalDateTime::now();
		if let Some(configuration) = configurator.get("server.strategies") {
			if let Some(list) = configuration.clone().into_vec() {
				let default = String::from("");
				for config in list {
					let mut id = &default;
					let mut classname = &default;
					if let Yaml::Hash(conf) = &config {
						if let Some(Yaml::String(v)) = get_yaml_key("class", conf) {
							classname = v;
						} else {
							return Err(OpenHemsError::new(format!(
								"Missing key 'id' for strategy."
							)));
						}
						if let Some(Yaml::String(v)) = get_yaml_key("id", conf) {
							id = v;
						} else {
							return Err(OpenHemsError::new(format!(
								"Missing key 'id' for strategy."
							)));
						}
						match classname.to_lowercase().as_str() {
							"offpeak" => {
								strategies.push(OffPeakStrategy::new(network, &id, conf)?);
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
		let server = Server{
			network: network,
			loopdelay: loopdelay,
			strategies: strategies,
			cycleid: 0,
			allowsleep: allowsleep,
			now: now,
			inoverloadmode: false
		};
		// network.set_server(&server);
		Ok(server)
	}
	pub fn loop1(&mut self, now:LocalDateTime) {
		self.now = now;
		
		/* if let Err(err) = self.network.update() {
			log::error!("Fail update network");
		} */
		let mut sleep_duration = self.loopdelay;
		for strategy in &self.strategies {
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