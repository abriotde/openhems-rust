use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use chrono::{DateTime, Local};
use hashlink::linked_hash_map::LinkedHashMap;
use yaml_rust2::Yaml;
use crate::error::ResultOpenHems;
use crate::network::Network;
use crate::node::Node;
use crate::offpeak_strategy::EnergyStrategy;
use crate::time;

// #[derive(Clone)]
pub struct SolarNoSellStrategy {
	id: String,
	ratio: f32,
	margin: f32,
	cycle_duration: u32,
	cycle_nb: i32,
	ref_coefficient: f32,
	coefs: Vec<f32>, // List of last cycle_nb coefs
	network: Rc<RefCell<Network>>,
	next_eval_date: DateTime<Local>,
	eval_frequency: chrono::Duration,
	deferables: HashMap<String, u32>, // List of devices to switch on/off
}

impl<'a, 'b:'a> EnergyStrategy for SolarNoSellStrategy {
	fn get_id(&self) -> &str {
		&self.id
	}
	fn get_nodes(&self) -> &Vec<Box<dyn Node>> {
		todo!();
	}
	fn update_network(&mut self, now:DateTime<Local>) -> ResultOpenHems<u64> {
		self.check(now)?;
		let cycle_duration = 30;
		self.apply(cycle_duration, now)
	}
}

impl SolarNoSellStrategy {
	pub fn new(network:Rc<RefCell<Network>>, id:&str, _config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<SolarNoSellStrategy> {
		Ok(SolarNoSellStrategy {
			id: id.to_string(),
			network: network,
			margin: 100000.0,
			cycle_duration: 0,
			cycle_nb: 0,
			coefs: Vec::new(),
			next_eval_date: time::MIN_DATETIME.clone(),
			eval_frequency: chrono::Duration::seconds(60),
			ratio: 1.0,
			ref_coefficient: 0.0,
			deferables: HashMap::new(),
		})
	}
	fn apply(&mut self, cycle_duration:u32, now:DateTime<Local>) -> ResultOpenHems<u64> {
		/*
		Called on each loop to switch on/off devices.
		Switch on devices if production > consommation + X * consommationDevice
		Switch off devices if production < consommation - (1-X) * consommationDevice
		Chances are we avoid ping-pong effect because when start device, we use max power,*
		  but usually the real power is lower, and it's this we use to switch off
		*/
		// logger.debug("SolarNoSellStrategy.apply()")
		let mut power_margin = 0.0;
		{
			let network = self.network.borrow();
			let consumption = network.get_current_power("all")?;
			let consumption_battery = network.get_current_power("battery")?;
			let production_solarpanel = network.get_current_power("solarpanel")?;
			power_margin = production_solarpanel - consumption + consumption_battery;
		}
		if power_margin>self.margin {
			if self.switch_on_devices(&mut power_margin)? {
				let dt = ((cycle_duration as f32)/5.0).max(3.0);
				return Ok(dt as u64);
			}
		} else if power_margin<self.margin {
			if self.switch_off_devices(&mut power_margin)? {
				let dt = ((cycle_duration as f32)/5.0).max(3.0);
				return Ok(dt as u64);
			}
		}
		// TODO : Return short timeout if we switch on a device {
		//  to quicly react if it's not enough (or too much {
		//  (more chances are the state will evolv after).
		Ok(100000)
	}
	fn eval(&self) {
		/*
		Useless in that case.
		*/
		// logger.debug("EnergyStrategy.eval()")
	}
	fn check(&mut self, now:DateTime<Local>) -> ResultOpenHems<()> {
		/*
		Check and eval if necessary
		- EMHASS optimization
		- power margin
		- conformity to EMHASS plan
		*/
		// self.logger.debug("EnergyStrategy.check()")
		if now>self.next_eval_date // || self.update_deferables()
		{
			// logger.debug("EnergyStrategy.check() : eval")
			self.eval();
			self.next_eval_date = now + self.eval_frequency;
		}
		Ok(())
	}

	fn switch_on_devices(&mut self, power_margin:&mut f32) -> ResultOpenHems<bool> {
		// """
		// Switch on devices if production > consommation + X * consommationDevice
		// Can switch on many devices if there is enought power powerMargin
		// """
		assert!(*power_margin>self.margin);
		let network = self.network.borrow();
		for node in network.get_all_switch("") {
			if node.is_on()? {
				continue;
			}
			// production > consommation + X * consommationDevice - powerMargin
			//  = (production - consommation) > X * consommationDevice
			//  = powerMargin  > X * consommationDevice
			// powerMargin+(((ratio-1)²-4)/4)*consommationDevice-ratio*margin>0
			let node_power= node.get_max_power();
			let coef = *power_margin + (((self.ratio-1.0).powi(2)-4.0)/4.0)*node_power - self.ratio*self.margin;
			if coef<=0.0 {
				continue;
			}
			self.cycle_nb = if self.cycle_nb>=0 { self.cycle_nb+1 } else { 1 };
			let c = self.cycle_nb;
			self.coefs.push(coef);
			log::info!("SolarNoSellStrategy: coef+={coef}");
			let sum: f32 = self.coefs.iter().sum();
			if c>=self.cycle_duration as i32
					|| sum>self.ref_coefficient {
				if let Err(err) = node.switch(true) {
					let message = format!("SolarNoSellStrategy : Fail to switch on device '{}' : {}", node.get_id(), err.message);
					log::error!("{}", message);
					network.notify(&message)?;
				} else {
					*power_margin -= node_power;
					if *power_margin<=0.0 {
						return Ok(true);
					}
				}
			}
		}
		Ok(false)
	}

	fn switch_off_devices(&mut self, power_margin:&mut f32) -> ResultOpenHems<bool> {
		// """
		// Switch off devices if production < consommation - (1-X) * consommationDevice
		// Can switch off many devices if there is enought power powerMargin
		// """
		assert!(*power_margin<self.margin);
		let mut network = self.network.borrow_mut();
		for node in network.get_all_switch_mut("all") {
			if !node.is_on()? {
				continue;
			}
			// production < consommation - (1-X) * consommationDevice
			//  = (production - consommation) < (X-1) * consommationDevice
			// Solution with coef between -1 and 1 : X = - (((ratio-1)²-4)/4)
			// powerMargin+(1+(((ratio-1)²-4)/4))*consommationDevice-ratio*margin<0
			let node_power= node.get_current_power()?;
			let coef = *power_margin + (1.0+((self.ratio-1.0).powi(2)-4.0)/4.0)*node_power - self.ratio*self.margin;
			if coef>=0.0 {
				continue;
			}
			self.cycle_nb = if self.cycle_nb<=0 { self.cycle_nb-1 } else { -1 };
			let c = -1*self.cycle_nb;
			self.coefs.push(coef);
			log::info!("SolarNoSellStrategy: coef+={coef}");
			let sum:f32 = self.coefs.iter().sum();
			if c>=self.cycle_duration as i32
					|| sum>self.ref_coefficient {
				if let Err(err) = node.switch(false) {
					let message = format!("SolarNoSellStrategy : Fail to switch off device '{}' : {}", node.get_id(), err.message);
					log::error!("{}", message);
					// network.notify(&message)?;
				} else {
					*power_margin += node_power;
					if *power_margin>=0.0 {
						return Ok(true);
					}
				}
			}
		}
		Ok(false)
	}

	fn update_deferables(&mut self) -> bool {
		/*
		Update scheduled devices list.
		It evolved if a node as been manually added
		 or scheduled duration have been manually changed
		 or duration evolved due to switched on
		Return true if schedule has been updated
		"""
		*/
		// self.logger.debug("EnergyStrategy.updateDeferables()")
		let mut update = false;
		let mut deferables = HashMap::new();
		let mut network = self.network.borrow_mut();
		// let mut to_drop = Vec::new();
		for node in network.get_all_switch_mut("all") {
			let node_id = node.get_id().to_string();
			let schedule = node.get_schedule();
			let is_scheduled = schedule.is_scheduled();
			if let Some(duration) = self.deferables.get(&node_id) {
				if !is_scheduled {
					update = true;
					// to_drop.push(node_id);
				} else {
					deferables.insert(node_id, schedule.get_duration());
					if *duration != schedule.get_duration() {
						update = true;
					}
				}
			} else {
				if is_scheduled {
					// Add a new deferrable
					deferables.insert(node_id, schedule.get_duration());
					update = true;
				}
			}
		}
		if update {
			self.deferables = deferables;
		}
		update
	}
}
