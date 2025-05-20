use std::{cell::RefCell, cmp::min, fmt::Debug, rc::Rc, sync::Arc, thread::sleep, time::Duration};
use chrono::{DateTime, Local, MappedLocalTime, NaiveDate, NaiveDateTime};
use yaml_rust2::Yaml;
use crate::{
	configuration_manager::ConfigurationManager, error::{OpenHemsError, ResultOpenHems}, network::Network, offpeak_strategy::{EnergyStrategy, OffPeakStrategy}, time, utils::get_yaml_key, web::AppState
};

pub trait DecrementTime {
	fn decrement_time(&mut self, duration:u32) -> ResultOpenHems<bool>;
}




// #[derive(Clone)]
pub struct Server {
	pub network: Rc<RefCell<Network>>,
	loopdelay: u64,
	strategies: Vec<OffPeakStrategy>,
	cycleid: u32,
	_allowsleep: bool,
	now: DateTime<Local>,
	_inoverloadmode: bool,
	_errors: Vec<String>,
	app_state: Arc<AppState>,
}
impl<'a> Debug for Server {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "Server(loopdelay:{}, cycleid:{}) On \n{:?}\n", self.loopdelay, self.cycleid, self.network)
	}
}
impl<'a> Server {
	pub fn new(configurator: &ConfigurationManager) -> ResultOpenHems<Server> {
		let allowsleep = true;
		let now = time::MIN_DATETIME.clone();
		let loopdelay = configurator.get_as_int("server.loopDelay") as u64;
		let strategies = Vec::new();
		let hems_server = Server {
			network: Rc::new(RefCell::new(Network::new(configurator)?)),
			loopdelay: loopdelay,
			strategies: strategies,
			cycleid: 0,
			_allowsleep: allowsleep,
			now: now,
			_inoverloadmode: false,
			_errors: Vec::new(),
			app_state: Arc::new(AppState::new()),
		};
		Ok(hems_server)
	}
	pub fn init(&mut self, configurator: &ConfigurationManager, appstate:&mut AppState) -> ResultOpenHems<()> {
		let mut network = self.network.borrow_mut();
		network.set_nodes(configurator, appstate);
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
	pub fn loop1(&mut self, now:DateTime<Local>, duration:u32) {
		log::info!("Server::loop1({:?}, {})", now, duration);
		self.now = now;
		if duration> 0 {
			self.app_state.decrement_time(duration).unwrap();
			/* for d in self.decrement_time.iter_mut() {
				if let Err(err) = d.decrement_time(duration) {
					log::error!("Fail decrement time : {}", err.message);
				}
			} */
		}
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
	pub fn run(&mut self, data: Arc<AppState>) {
		self.app_state = data;
		let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    	let r = running.clone();
		ctrlc::set_handler(move || {
			r.store(false, std::sync::atomic::Ordering::SeqCst);
		}).expect("Failed to set Ctrl+C handler");
		log::info!("Run OpenHEMS core server with loop-delay={}", self.loopdelay);
		let mut duration = 0;
		let loopdelay = Duration::from_secs(self.loopdelay);
		while running.load(std::sync::atomic::Ordering::SeqCst) {
			let now = Local::now();
			let nextloop = now + loopdelay;
			self.loop1(now, duration);
			let t = Local::now();
			if t<nextloop {
				let secs = nextloop - t;
				duration = self.loopdelay as u32;
				log::info!("Sleep for {} seconds.", secs.num_seconds());
				sleep(secs.to_std().unwrap());
			} else if t>nextloop {
				let secs = (t - nextloop).as_seconds_f32();
				duration = self.loopdelay as u32 + secs as u32;
				log::warn!("Missing {secs} seconds for the loop.");
			}
		}
	}

}