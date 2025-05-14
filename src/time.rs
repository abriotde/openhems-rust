use datetime::{LocalTime, LocalDateTime};
use core::fmt;
use iso8601;
use regex::Regex;
use lazy_static::lazy_static;
use yaml_rust2::Yaml;
use crate::error::{OpenHemsError, ResultOpenHems};

lazy_static!{
	pub static ref REGEX_HOUR_MIN_SEC: Regex = Regex::new(r"^([0-9]+):|h([0-9]+):|m([0-9]+)$").unwrap();
	pub static ref REGEX_HOUR_MIN: Regex = Regex::new(r"^([0-9]+):|h([0-9]+)").unwrap();
	pub static ref REGEX_HOUR: Regex = Regex::new(r"^([0-9]+)h?$").unwrap();
	pub static ref MIN_DATETIME: LocalDateTime = LocalDateTime::at(0);
 }

fn from_openhems_str(input: &str) -> ResultOpenHems<LocalTime> {
	if let Ok(fields) = iso8601::time(input) {
		return LocalTime::hm(
				i8::try_from(fields.hour).unwrap(), 
				i8::try_from(fields.minute).unwrap()
			)
			.map_err(|e| OpenHemsError::new(
				format!("from_openhems_str(iso8601::time) {}",e.to_string())
			));
	}
	if let Some(caps) = REGEX_HOUR_MIN_SEC.captures(input) {
        let hour = caps[1].parse::<i8>().unwrap();
        let min = caps[2].parse::<i8>().unwrap();
        let sec = caps[3].parse::<i8>().unwrap();
		return LocalTime::hms(hour, min, sec)
			.map_err(|e| OpenHemsError::new(
				format!("from_openhems_str(HOUR_MIN_SEC) {}",e.to_string())
			));
    }
	if let Some(caps) = REGEX_HOUR_MIN.captures(input) {
        let hour = caps[1].parse::<i8>().unwrap();
        let min = caps[2].parse::<i8>().unwrap();
		return LocalTime::hms(hour, min, 0)
			.map_err(|e| OpenHemsError::new(
				format!("from_openhems_str(HOUR_MIN) {}",e.to_string())
			));
    }
	if let Some(caps) = REGEX_HOUR.captures(input) {
        let hour = caps[1].parse::<i8>().unwrap();
		return LocalTime::hms(hour, 0, 0)
			.map_err(|e| OpenHemsError::new(
				format!("from_openhems_str(HOUR) {}",e.to_string())
			));
    }

	Err(OpenHemsError::new(format!("Fail parse {input}")))
}

#[derive(Debug, Clone, Copy)]
pub struct HoursRange {
	pub start: LocalTime,
	pub end: LocalTime,
	pub cost: f32
}
impl HoursRange {
	pub fn from(config:&Yaml, default_cost:f32) -> ResultOpenHems<HoursRange> {
		let mut ret = HoursRange {
			start: LocalTime::midnight(),
			end: LocalTime::midnight(),
			cost: default_cost,
		};
		let mut ko = true;
		match config {
			Yaml::String(val) => {
				ko = !Self::fill_with_split(&val, &mut ret)?;
			}
			Yaml::Array(list) => {
				if let Yaml::String(val) = &list[0] {
					ko = !Self::fill_with_split(val, &mut ret)?;
					if (!ko) && list.len()==2 {
						ret.cost = Self::get_cost(&list[1], default_cost)?;
					}
				}
				if (!ko) && list.len()>=2 {
					if let Yaml::String(val) = &list[0] {
						ret.start = from_openhems_str(&val)?;
					}
					if let Yaml::String(val) = &list[1] {
						ret.end = from_openhems_str(&val)?;
						ko = false;
					}
					if list.len()==3 {
						ret.cost = Self::get_cost(&list[2], default_cost)?;
					}
				}
			}
			_ => {
				return Err(OpenHemsError::new(format!("HoursRange::from() : Invalid configuration")));
			}
		}
		if ko {
			return Err(OpenHemsError::new(format!("HoursRange::from() : Missing something.")));
		}
		Ok(ret)
	}
	fn get_cost(config:&Yaml, default_cost:f32) -> ResultOpenHems<f32> {
		if let Yaml::Real(val) = config {
			Ok(val.parse().unwrap())
		} else if let Yaml::Integer(val) = config {
			Ok(*val as f32)
		} else {
			Ok(default_cost)
		}
	}
	fn fill_with_split(val:&str, ret:&mut HoursRange) -> ResultOpenHems<bool> {
		let mut ko = true;
		if val.contains("-") {
			let mut split = val.split('-');
			let mut i = 0;
			while let Some(a) = split.next() {
				if i==0 {
					ret.start = from_openhems_str(a)?;
					i = 1;
				} else {
					ret.end = from_openhems_str(a)?;
					ko = false;
				}
			}
			Ok(!ko)
		} else {
			Ok(false)
		}
	}
	pub fn get_end(&self, now:LocalDateTime) -> LocalDateTime {
		let start = now.time();
		let nbseconds = HoursRanges::get_timetowait(&start, &self.end);
		// println!("get_end() : Add {nbseconds} seconds. {start:?} - {:?}", self.end);
		now.add_seconds(nbseconds)
	}
	pub fn get_start(&self, now:LocalDateTime) -> LocalDateTime {
		let end = now.time();
		let nbseconds = HoursRanges::get_timetowait(&self.start, &end);
		now.add_seconds(-nbseconds)
	}
}

pub trait HoursRangesCallback {
	fn callback(&self); // , ranges: &HoursRanges);
}

// #[derive(Debug, Clone)]
pub struct HoursRanges {
	index: u32,
	ranges:Vec<HoursRange>,
	min_cost: f32,
	range_end: LocalDateTime,
	timeout: Option<LocalDateTime>,
	time_start: Option<LocalDateTime>,
	timeout_callback: Option<Box<dyn HoursRangesCallback>>,
}
impl fmt::Debug for HoursRanges {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let _ = write!(f, "HoursRanges(");
		for range in self.ranges.iter() {
        	let _ = write!(f, ", {:?}", range);
		}
		write!(f, ")")
    }
}
impl Clone for HoursRanges {
	fn clone(&self) -> Self {
		let mut ranges = Vec::new();
		for range in self.ranges.iter() {
			ranges.push(range.clone());
		}
		HoursRanges {
			index: self.index,
			ranges: ranges,
			min_cost: self.min_cost,
			range_end: self.range_end,
			timeout: self.timeout,
			time_start: self.time_start,
			timeout_callback: None, // TODO
		}
	}
}
fn fmt(a:&LocalTime) -> String {
	let mut seconds = a.to_seconds();
	let  hours = seconds/3600;
	seconds -= hours*3600;
	let  min = seconds/60;
	format!("{hours}h{min}")
}
	
impl fmt::Display for HoursRanges {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let _ = write!(f, "HoursRanges(");
		if self.ranges.len()>0 {
			let mut lastend = self.ranges[0].end;
			for range in self.ranges.iter() {
				let _ = write!(f, ", {} ${}", fmt(&range.start), range.cost);
				lastend = range.end;
			}
			write!(f, "{})", fmt(&lastend))
		} else {
			write!(f, ")")
		}
    }
}

impl HoursRanges {
	pub fn from(hoursranges_list:&Yaml, 
			time_start:Option<LocalDateTime>, timeout:Option<LocalDateTime>,
			timeout_callback:Option<Box<dyn HoursRangesCallback>>,
			default_cost:f32, outrange_cost:f32
	) -> ResultOpenHems<HoursRanges> {
		let mut ranges = Vec::new();
		match hoursranges_list {
			Yaml::String(_) => {
				ranges.push(HoursRange::from(hoursranges_list, default_cost)?);
			}
			Yaml::Array(list) => {
				for range in list {
					let range = HoursRange::from(range, default_cost);
					// TODO : If errors, maybe is it a HoursRange as list
					ranges.push(range?);
				}
			}
			_ => {}
		}
		let mut ret = HoursRanges {
			index: 0,
			ranges: ranges,
			min_cost: std::f32::MAX,
			range_end: *MIN_DATETIME,
			timeout: timeout,
			time_start: time_start,
			timeout_callback: timeout_callback
		};
		ret.fill_ranges(outrange_cost)?;
		Ok(ret)
	}
	fn fill_ranges(&mut self, outrange_cost:f32) -> ResultOpenHems<()> {
		if self.ranges.len()==0 {
			/* let midnight = LocalTime::midnight();
			ranges.insert(HoursRange{
				start: midnight,
				end: midnight,
				cost: outrange_cost
			}) */
			return Ok(());
		}
		self.ranges.sort_by(|a, b| {
			a.start.to_seconds().cmp(&b.start.to_seconds())
		});
		let firstbegin = self.ranges[0].end;
		let mut lastend = self.ranges[self.ranges.len()-1].end;
		let mut addedranges: Vec<HoursRange> = Vec::new();
		for range in self.ranges.iter() {
			// print("range:", begin, end, "lastEnd:", lastEnd)
			if lastend.to_seconds()<range.start.to_seconds() {
				addedranges.push(
					HoursRange {
						start:lastend,
						end: range.start,
						cost: outrange_cost
				})
			} else if range.start.to_seconds()<lastend.to_seconds() { // Should be equal
				return Err(OpenHemsError::new(
					format!("HoursRanges : ranges are crossing : {:?} < {:?}", range.start, lastend)
				))
			}
			if range.cost<self.min_cost {
				self.min_cost = range.cost;
			}
			lastend = range.end;
		}
		// Close the cycle from end to the begeining
		if lastend.to_seconds()!=firstbegin.to_seconds() {
			self.ranges.push(HoursRange{
				start: lastend,
				end: firstbegin,
				cost: outrange_cost
			});
		}
		if addedranges.len()>0 {
			self.ranges.extend(addedranges.iter());
		}
		self.ranges.sort_by(|a, b| {
			a.start.to_seconds().cmp(&b.start.to_seconds())
		});
		Ok(())
	}
	pub fn is_offpeak(&self, range:&HoursRange) -> bool {
		self.min_cost==range.cost
	}
	pub fn get_timetowait(from:&LocalTime, to:&LocalTime) -> i64 {
		// "10:00", "02:00"
		let from_s = from.to_seconds();
		let to_s = to.to_seconds();
		let mut dt = to_s-from_s;
		if dt<0 {
			dt += 24*3600;
		}
		// println!("get_timetowait({:?} - {:?}) : {to_s}-{from_s} = {dt}", from , to);
		dt
		// print("getTimeToWait(",self.time,", ",nextTime,") = ",wait)
	}
	pub fn check_range(&self, now:LocalDateTime) -> ResultOpenHems<&HoursRange> {
		// Check range validity of this hoursRange
		if let Some(time) = self.time_start {
			if now<time {
				return Err(OpenHemsError::new(format!("")));
			}
		}
		if let Some(time) = self.timeout {
			if now>time {
				if let Some(cb) = &self.timeout_callback {
					cb.callback();
				}
				// return Err(OpenHemsError::new(format!("")));
			}
		}
		//TODO : return self._timeoutCallBack.getHoursRanges(nowDatetime, attime).checkRange(nowDatetime, attime)
		// print("OffPeakStrategy.checkRange(",now,")")
		// # This has no real signification but it's usefull and the most simple way
		let mut time2nextrange = 3600*24; // = 24h = a full day
		let mut currange = &self.ranges[0];
		let timenow = now.time();
		for hoursrange in &self.ranges {
			let  time = &hoursrange.end;
			let wait= Self::get_timetowait(&timenow, time);
			if wait<time2nextrange {
				currange = hoursrange;
				time2nextrange = wait;
			}
		}
		Ok(currange)
	}
}

#[cfg(test)]
mod tests {
	use datetime::{DatePiece, LocalDate, Month};
use yaml_rust2::YamlLoader;
    use super::*;

    #[test]
    fn test_time_hoursranges() -> ResultOpenHems<()> {
		let configs = YamlLoader::load_from_str("[\"22h-6h\"]").unwrap();
		let config = &configs[0];
		let ranges = HoursRanges::from(config, 
			None, None, None, 0.0, 1.0)?;
		println!("{ranges}");
		let date = LocalDate::ymd(2025, Month::April, 28).unwrap();
		let time = LocalTime::hm(11, 00).unwrap();
		let now = LocalDateTime::new(date, time);
		let range = ranges.check_range(now)?;
		assert_eq!(ranges.is_offpeak(&range), false);
		Ok(())
    }
	// mael@allianz.fr
}