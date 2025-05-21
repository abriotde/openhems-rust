use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use arrayvec::ArrayString;
use crate::error::OpenHemsError;

use crate::home_assistant_api::HomeAssistantAPI;
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
impl FeederOutType<bool> for bool {
	fn default() -> bool {
		true
	}
}
#[derive(Clone, Debug)]
pub enum Feeder<T:FeederOutType<T>+Clone> {
	Source(SourceFeeder<T>),
	Const(ConstFeeder<T>)
}
impl<'a> Feeder<bool> {
	pub fn get_value(&self) -> ResultOpenHems<bool> {
		match self.clone() {
			Feeder::Source(mut feeder) => {
				feeder.get_value()
			}
			Feeder::Const(mut feeder) => {
				feeder.get_value()
			}
		}
	}
}

// #[derive(Debug, Clone)] // , Clone implemented manually
#[derive(Debug, Clone)]
pub struct SourceFeeder<T:FeederOutType<T>+Clone> {
	nameid: ArrayString<64>, // Home Assistant  entity id are long (sensor.lixee_zlinky_tic_puissance_apparente)
	source: Rc<RefCell<HomeAssistantAPI>>,
	cycle_id:u32,
	value: T
}
/* impl<'a, 'b:'a, T:FeederOutType<T>+Clone> Clone for SourceFeeder<T> {
    fn clone(&self) -> SourceFeeder<'a, T> {
        SourceFeeder {
			nameid: self.nameid,
			source: self.source,
			cycle_id: self.cycle_id,
			value: self.value.clone(),
		}
    }
} */
impl<'a, 'b:'a, T:FeederOutType<T>+Clone> SourceFeeder<T> {
	pub fn new(updater:Rc<RefCell<HomeAssistantAPI>>, entity_id:&str) -> ResultOpenHems<SourceFeeder<T>> {
		let nameid = ArrayString::from(entity_id)
			.map_err(|message| OpenHemsError::new(
				format!("Entity id '{entity_id}' is too long : {}", message.to_string())
			))?;
		{
			let mut updater2 = updater.borrow_mut();
			updater2.register_entity(entity_id);
		}
		Ok(SourceFeeder {
			nameid: nameid,
			source: updater,
			cycle_id: 0,
			value: T::default()
		})
	}
	pub fn get_nameid(&self) -> &ArrayString<64> {
		&self.nameid
	}
	pub fn switch(&self, nameid:&str, on:bool) -> ResultOpenHems<bool>{
		let updater = self.source.borrow();
		updater.switch(nameid, on)
	}
}
/* impl<'a, T:FeederOutType<T>+Clone> SourceFeeder<T> {
	pub fn get_value(&mut self) -> ResultOpenHems<T> {
		Ok(T::default())
	}
} */
macro_rules! get_value (
    ($t:ident, $f:ident) => (
#[must_use]
pub fn get_value(&mut self) -> ResultOpenHems<$t> {
	let source = self.source.borrow_mut();
	let cur_id = source.get_cycle_id();
	if self.cycle_id <= cur_id {
		self.value = source.$f(&self.nameid)?;
		self.cycle_id = cur_id;
	}
	Ok(self.value)
}
    );
);

impl<'a> SourceFeeder<i32> {
	get_value!(i32, get_entity_value_int);
}
impl<'a> SourceFeeder<f32> {
	get_value!(f32, get_entity_value_float);
}
impl<'a> SourceFeeder<String> {
	pub fn get_value(&mut self) -> ResultOpenHems<String> {
		let source = self.source.borrow_mut();
		let cur_id = source.get_cycle_id();
		if self.cycle_id <= cur_id {
			self.value = source.get_entity_value_str(&self.nameid)?;
			self.cycle_id = cur_id;
		}
		Ok(self.value.clone())
	}
}
impl<'a> SourceFeeder<bool> {
	get_value!(bool, get_entity_value_bool);
}

#[derive(Clone, Debug, Copy)]
pub struct ConstFeeder<T:FeederOutType<T>+Clone> {
	value: T
}
impl<T:FeederOutType<T>+Clone> ConstFeeder<T> {
	pub fn new(value:T) -> ConstFeeder<T> {
		ConstFeeder {
			value: value
		}
	}
	pub fn get_value(&mut self) -> ResultOpenHems<T> {
		Ok(self.value.clone())
	}
}

/* #[derive(Clone, Debug)]
pub struct GuessIsOnFeeder<T:FeederOutType<T>+Clone> {
	source: Feeder<Feeder<T>>
}
impl<'a, 'b:'a, T:FeederOutType<T>+Clone> Clone for GuessIsOnFeeder<T> {
    fn clone(&self) -> GuessIsOnFeeder<T> {
        GuessIsOnFeeder {
			source: self.source.clone()
		}
    }
}
impl Feeder for GuessIsOnFeeder<bool> {
	type Item = bool;
	fn get_value(&mut self) -> ResultOpenHems<bool> {
		self.source.get_value()
	}
} */
