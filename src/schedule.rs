use arrayvec::ArrayString;
use chrono::{DateTime, Local};
use json::JsonValue;
use crate::{error::{OpenHemsError, ResultOpenHems}, server::DecrementTime, time, web};

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
		format!("{{\"name\":\"{}\", \"duration\":{}, \"date\":\"{}\", \"timeout\":\"{}\"}}",
			self.nameid, self.get_duration(),
			self.get_timeout().format(web::DATE_FORMAT), self.get_timeout().format("%H:%M"))
	}
	pub fn update_from_json(&mut self, schedule_json:&JsonValue) -> ResultOpenHems<()> {
		if let JsonValue::Object(sch) = schedule_json {
			let mut update = false;
			let mut timeout = time::MIN_DATETIME.clone();
			let mut duration = 0;
			if let Some(d) = sch.get("duration") {
				if let Some(d1) = d.as_i32() {
					duration = d1 as u32;
					update = true;
				}
			}
			if let Some(date) = sch.get("timeout") {
				if let Some(d1) = date.as_str() {
					if let Ok(timeout_new) = chrono::NaiveTime::parse_from_str(d1, "%H:%M") {
						let now = Local::now();
						timeout = time::time2datetime(&timeout_new, &now);
						update = true;
					}
				}
			}
			if update {
				self.set_duration(duration);
				self.set_timeout(&timeout);
			} else {
				return Err(OpenHemsError::new(format!("Error parsing Schedule json : {}", schedule_json)));
			}
		}
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