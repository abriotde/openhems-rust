use core::time;

use arrayvec::ArrayString;
use chrono::{DateTime, Local};

use crate::error::ResultOpenHems;

pub struct Schedule {
	nameid: ArrayString<16>,
	duration:u32,
	timeout:DateTime<Local>,
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
		let json= format!("\"name\":\"{}\", \"duration\":{},timeout:\"{}\"",
			self.nameid, self.duration, self.timeout.format("%H:%M:%S"));
		Ok(())
	}
}