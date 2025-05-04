use arrayvec::ArrayString;
use crate::error::OpenHemsError;

use crate::{error::ResultOpenHems, home_assistant_api::HomeStateUpdater};

pub trait FeederOutType<T:Clone> {
	fn default() -> T;
}
impl FeederOutType<i32> for i32 {
	fn default() -> i32 {
		0
	}
}
impl FeederOutType<f32> for f32 {
	fn default() -> f32 {
		0.0
	}
}
impl FeederOutType<String> for String {
	fn default() -> String {
		String::from("")
	}
}
pub trait Feeder {
	type Item:FeederOutType<Self::Item>+Clone;
	fn get_value(&mut self) -> ResultOpenHems<Self::Item>;
}

#[derive(Debug)]
pub struct SourceFeeder<'a, Updater:HomeStateUpdater, T:FeederOutType<T>+Clone> {
	nameid: ArrayString<64>, // Home Assistant  entity id are long (sensor.lixee_zlinky_tic_puissance_apparente)
	source: &'a Updater,
	cycle_id:u32,
	value: T
}
impl<'a, Updater:HomeStateUpdater, T:FeederOutType<T>+Clone> Clone for SourceFeeder<'a, Updater, T> {
    fn clone(&self) -> SourceFeeder<'a, Updater, T> {
        SourceFeeder {
			nameid: self.nameid,
			source: self.source,
			cycle_id: self.cycle_id,
			value: self.value.clone(),
		}
    }
}
impl<'a, Updater:HomeStateUpdater, T:FeederOutType<T>+Clone> SourceFeeder<'a, Updater, T> {
	pub fn new(updater:&'a Updater, entity_id:&str) -> ResultOpenHems<SourceFeeder<'a, Updater, T>> {
		let nameid = ArrayString::from(entity_id)
			.map_err(|message| OpenHemsError::new(
				format!("Entity id '{entity_id}' is too long : {}", message.to_string())
			))?;
		Ok(SourceFeeder {
			nameid: nameid,
			source: updater,
			cycle_id: 0,
			value: T::default()
		})
	}
}
impl<'a, Updater:HomeStateUpdater> Feeder for SourceFeeder<'a, Updater, i32> {
	type Item = i32;
	fn get_value(&mut self) -> ResultOpenHems<Self::Item> {
		if self.cycle_id <= self.source.get_cycle_id() {
			self.value = self.source.get_entity_value_int(&self.nameid)?
		}
		Ok(self.value)
	}
}
impl<'a, Updater:HomeStateUpdater> Feeder for SourceFeeder<'a, Updater, f32> {
	type Item = f32;
	fn get_value(&mut self) -> ResultOpenHems<Self::Item> {
		if self.cycle_id <= self.source.get_cycle_id() {
			self.value = self.source.get_entity_value_float(&self.nameid)?
		}
		Ok(self.value)
	}
}
impl<'a, Updater:HomeStateUpdater> Feeder for SourceFeeder<'a, Updater, String> {
	type Item = String;
	fn get_value(&mut self) -> ResultOpenHems<Self::Item> {
		if self.cycle_id <= self.source.get_cycle_id() {
			self.value = self.source.get_entity_value_str(&self.nameid)?
		}
		Ok(self.value.clone())
	}
}

#[derive(Clone, Debug)]
pub struct ConstFeeder<T:FeederOutType<T>+Clone> {
	value: T
}
impl<T:FeederOutType<T>+Clone> Feeder for ConstFeeder<T> {
	type Item = T;
	fn get_value(&mut self) -> ResultOpenHems<T> {
		Ok(self.value.clone())
	}
}
