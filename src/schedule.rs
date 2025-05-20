use core::time;

use arrayvec::ArrayString;
use chrono::{DateTime, Local};
use yaml_rust2::Yaml;

use crate::{error::ResultOpenHems, server::DecrementTime};

#[derive(Clone, Debug)]
pub struct Schedule {
	nameid: ArrayString<16>,
	duration:u32,
	timeout:DateTime<Local>,
}

impl DecrementTime for Schedule {
	fn decrement_time(&mut self, duration:u32) -> ResultOpenHems<bool> {
		log::debug!("Schedule::decrement_time({})", duration);
		let mut continu = false;
		if self.duration>0 {
			if duration>=self.duration {
				self.duration = 0;
			} else {
				self.duration -= duration;
				log::debug!("Schedule::decrement_time() last {} seconds", self.duration);
				continu = true;
			}
		}
		Ok(continu)
	}
}

impl Schedule {
	pub fn new(nameid:&ArrayString<16>) -> Schedule {
		Schedule {
			nameid: nameid.clone(),
			duration: 0,
			timeout:Local::now(),
		}
	}
	pub fn to_json(&self) -> String {
		let json= format!("\"name\":\"{}\", \"duration\":{},timeout:\"{}\"",
			self.nameid, self.duration, self.timeout.format("%H:%M:%S"));
		json
	}
	pub fn update_from_json(&mut self, json:&str) -> ResultOpenHems<()> {
		Ok(())
	}
	pub fn is_scheduled(&self) -> bool {
		let ok = self.duration>0;
		if ok {
			log::debug!("Schedule::is_scheduled() for {} seconds", self.duration);
		}
		ok
	}
	pub fn set_duration(&mut self, duration:u32) {
		self.duration = duration;
	}
	pub fn set_timeout(&mut self, timeout:&DateTime<Local>) {
		self.timeout = timeout.clone();
	}
	pub fn get_duration(&self) -> u32 {
		self.duration
	}
	pub fn get_timeout(&self) -> &DateTime<Local> {
		&self.timeout
	}
	pub fn get_name(&self) -> &str {
		&self.nameid
	}
}