use std::{cell::{RefCell, RefMut}, collections::HashMap, rc::Rc};
use reqwest;
use json::{self, JsonValue, object::Object};
use yaml_rust2::Yaml;
use core::fmt;
use std::io::Read;
use serde_json::json;
use crate::{
	cast_utility, configuration_manager::ConfigurationManager,
	error::{OpenHemsError, ResultOpenHems},
	feeder::{ConstFeeder, Feeder, SourceFeeder},
	network::Network, node::{self, Node, NodeBase, Switch}
};

pub trait HomeStateUpdater:Clone
{
    fn default() -> Self;
    fn notify(&self, message:&str) -> ResultOpenHems<bool> {
		print!("HomeStateUpdater.notify : {message}");
		Ok(true)
	}
    fn init_network(&mut self)-> ResultOpenHems<bool>;
    fn update_network(&mut self) -> ResultOpenHems<bool>;

	fn register_entity(&mut self, nameid:&str) -> bool;
	fn get_entity_value_int(&self, nameid:&str) -> ResultOpenHems<i32>;
	fn get_entity_value_float(&self, nameid:&str) -> ResultOpenHems<f32>;
	fn get_entity_value_str(&self, nameid:&str) -> ResultOpenHems<String>;
	fn get_entity_value_bool(&self, entity_id:&str) -> ResultOpenHems<bool>;

	// fn switch_on(&self) -> bool;
	// fn get_feeder(&self, value:&str, expectedType:&str);
	// fn get_publicpowergrid(&self, network:&'a Network, nameid:&str, node_conf:&HashMap<String, &Yaml>) -> ResultOpenHems<PublicPowerGrid>;
	// fn get_solarpanel(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
	// fn get_battery(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
    // fn get_network(&self) -> Network;
    fn get_cycle_id(&self) -> u32;
}

#[derive(Clone)]
pub struct HomeAssistantAPI {
    token: String,
    url: String,
    cached_ids: HashMap<String, JsonValue>,
	ha_elements: HashMap<String, JsonValue>,
	cycle_id:u32
}
impl<'a, 'b:'a, 'c:'b> fmt::Display for HomeAssistantAPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "HomeAssistantAPI()")
    }
}
impl<'a, 'b:'a, 'c:'b> fmt::Debug for HomeAssistantAPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "HomeAssistantAPI()")
    }
}

impl<'a> HomeAssistantAPI {
    pub fn new(configurator:&ConfigurationManager) -> ResultOpenHems<HomeAssistantAPI> {
		let url: String = configurator.get_as_str("api.url");
		let token = configurator.get_as_str("api.long_lived_token");
		let mut updater = HomeAssistantAPI {
			token: token,
			url: url,
			cached_ids: HashMap::new(),
			ha_elements: HashMap::new(),
			cycle_id: 0
		};
		updater.init_network()?;
		Ok(updater)
	}
	// TODO : post with data + Status...
	pub fn call_api(&self, url: &str, data: Option<serde_json::Value>) -> Result<JsonValue, OpenHemsError> {
		// let client = reqwest::Client::new();
		let client = reqwest::blocking::Client::new();
		let mut bearer = String::from("Bearer ");
		bearer.push_str(&self.token);
		let mut complete_url: String = String::from(&self.url);
		complete_url.push_str(url);
		log::info!("Call Home-Assistant API : {complete_url}");
		let request_builder;
		if let Some(mydata) = data {
			// let mut body = reqwest::blocking::Body::new(mydata);
			let request_body_string = serde_json::to_string(&mydata).unwrap();
			log::info!("   with data = {}", request_body_string);
			request_builder = client.post(&complete_url)
				.body(request_body_string);
		} else {
			request_builder = client.get(&complete_url);
		}
		let mut res = request_builder
			.header("Authorization", bearer)
			.header("content-type", "application/json")
			.send()
			.map_err(|message| OpenHemsError::new(
				format!("Fail call Home-Assistant API for url '{url}' : {}",message.to_string())
			))?;
		let mut body = String::new();
		let _ = res.read_to_string(&mut body);
		// println!("Status: {}", a.status());
		// println!("Headers:\n{:#?}", a.headers());
		// println!("Body:\n{}", body);
		log::debug!("Call Home-Assistant API : {complete_url} : Ok");
		json::parse(&body)
			.map_err(|message| OpenHemsError::new(
				format!("Call Home-Assistant API for {url} : Fail parse '{body}' : {}", message.to_string())
			))
	}
	pub fn get_feeder_const_int(node_conf:&HashMap<String, &Yaml>, key:&str, default_value:i32) -> i32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_int(val)
		} else {
			default_value
		}
	}
	pub fn get_feeder_const_str(node_conf:&HashMap<String, &Yaml>, key:&str, default_value:&str) -> String {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_str(val)
		} else {
			default_value.to_string()
		}
	}
	pub fn get_feeder_const_float(node_conf:&HashMap<String, &Yaml>, key:&str, default_value:f32) -> f32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_float(val)
		} else {
			default_value
		}
	}
	pub fn get_feeder_float(updater:Rc<RefCell<HomeAssistantAPI>>, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:f32) -> ResultOpenHems<SourceFeeder<f32>> {
		if let Some(val) = node_conf.get(key) {
			if let Yaml::String(entity_id) = val {
				let updater2 = updater.borrow_mut();
				if updater2.ha_elements.contains_key(entity_id) {
					drop(updater2);
					SourceFeeder::new(Rc::clone(&updater), entity_id)
				} else {
					Err(OpenHemsError::new(format!("No  key '{key}'")))
				}
			} else {
				Err(OpenHemsError::new(format!("No  key '{key}'")))
			}
		} else {
			Err(OpenHemsError::new(format!("No  key '{key}'")))
		}
	}
	pub fn get_feeder_bool(updater:Rc<RefCell<HomeAssistantAPI>>, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:bool) -> ResultOpenHems<SourceFeeder<bool>> {
		if let Some(val) = node_conf.get(key) {
			if let Yaml::String(entity_id) = val {
				let updater2 = updater.borrow_mut();
				if updater2.ha_elements.contains_key(entity_id) {
					drop(updater2);
					// <HomeAssistantAPI, f32>
					SourceFeeder::new(Rc::clone(&updater), entity_id)
				} else {
					Err(OpenHemsError::new(format!("No  key '{key}'")))
				}
			} else {
				Err(OpenHemsError::new(format!("No  key '{key}'")))
			}
		} else {
			Err(OpenHemsError::new(format!("No  key '{key}'")))
		}
	}
	pub fn get_nodebase(updater:Rc<RefCell<HomeAssistantAPI>>, nameid:&str, node_conf:&HashMap<String, &Yaml>) -> ResultOpenHems<NodeBase> {
		let max_power = HomeAssistantAPI::get_feeder_const_float(node_conf, "maxPower", 0.0);
		let min_power = HomeAssistantAPI::get_feeder_const_float(node_conf, "minPower", 0.0);
		let current_power = HomeAssistantAPI::get_feeder_float(Rc::clone(&updater), node_conf, "currentPower", 0.0)?;
		let is_on:Feeder<bool>;
		if let Ok(source_feeder) = HomeAssistantAPI::get_feeder_bool(Rc::clone(&updater), node_conf, "isOn", false) {
			is_on = Feeder::Source(source_feeder);
		} else {
			is_on = Feeder::Const(ConstFeeder::new(true));
		}
		let node = node::get_nodebase(nameid, max_power, min_power, current_power, is_on)?;
		Ok(node)
	}
	pub fn get_entity_value(&self, entity_id:&str) -> ResultOpenHems<&JsonValue> {
		if self.cached_ids.contains_key(entity_id) {
			Ok(self.cached_ids.get(entity_id).unwrap())
		} else {
			/* let mut url = String::from("/states/");
			url.push_str(entity_id);
			let value = self.call_api(&url, None)?;
			self.cached_ids.insert(entity_id.to_string(), value);
			if let Some(value) = self.cached_ids.get(entity_id) {
				Ok(value)
			} else { // Should be impossible
				Ok(&JsonValue::Null)
			} */
			 Err(OpenHemsError::new(format!("No entity '{entity_id}' found.")))
		}
	}
    pub fn init(&mut self, url: String, token: String) -> ResultOpenHems<bool> {
		self.url = url;
		self.token = token;
		self.init_network()
	}
	pub fn switch(&self, entity_id:&str, on:bool) -> ResultOpenHems<()> {
		let data = json!({
			"entity_id": entity_id
		});
		let expect = if on {"on"} else {"off"};
		log::info!("Switching '{entity_id}' to {expect}.");
		let url = format!("/services/switch/turn_{}", expect);
		if let JsonValue::Array(response) = self.call_api(&url, Some(data))? {
			if let Some(states) = response.get(0) {
				println!("States : {:?}", states);
			}
		}
		Ok(())
	}
}

macro_rules! get_entity_value_ (
    ($fname:ident, $t:ident, $f:ident) => (
#[must_use]
fn $fname(&self, entity_id:&str) -> ResultOpenHems<$t> {
	let v= self.get_entity_value(entity_id)?;
	if let Some(value)  = v.$f() {
		Ok(value)
	} else {
		let message = format!("Value can not be parsed as $name.");
		Err(OpenHemsError::new(message))
	}
}
    );
);

impl<'a, 'b:'a, 'c:'b> HomeStateUpdater for HomeAssistantAPI {
    fn default() -> Self {
		HomeAssistantAPI {
			token: "".to_string(),
			url: "".to_string(),
			cached_ids: HashMap::new(),
			ha_elements: HashMap::new(),
			cycle_id: 0
		}
	}
    fn init_network(&mut self)-> ResultOpenHems<bool> {
		let states = self.call_api("/states", None)?;
		let mut count = 0;
		if let JsonValue::Array(parsed_list) = states {
			for elem in parsed_list {
				if let JsonValue::Object(entity) = elem {
					let entity_id = entity.get("entity_id").unwrap().as_str().unwrap();
					// println!(" - {entity_id}");
					if let Some(state) = entity.get("state") {
						// println!("HomeAssistantAPI::init_network() : '{entity_id}' := {state:?}")
						self.ha_elements.insert(String::from(entity_id), state.clone());
					}
					count = count + 1;
				}
			}
		}
		println!("Count {count}");
		Ok(true)
	}
	fn update_network(&mut self) -> ResultOpenHems<bool> {
		self.cycle_id += 1;
		if self.cached_ids.len()==0 {
			log::warn!("HomeAssistantAPI.update_network() : No entities to update.")
		}
		let states = self.call_api("/states", None)?;
		let mut count = 0;
		if let JsonValue::Array(parsed_list) = states {
			for elem in parsed_list {
				if let JsonValue::Object(entity) = elem {
					let entity_id = entity.get("entity_id").unwrap().as_str().unwrap();
					// println!(" - {entity_id}");
					if self.cached_ids.contains_key(entity_id) {
						if let Some(state) = entity.get("state") {
							log::debug!("HomeAssistantAPI::update_network() : '{entity_id}' := {state:?}");
							self.cached_ids.insert(String::from(entity_id), state.clone());
						} else {
							log::warn!("No state for  : '{entity_id}' := {entity:?}");
						}
					}
					count = count + 1;
				}
			}
		}
		log::debug!("update_network() : {} keys.", count);
		Ok(true)
	}
	fn notify(&self, message: &str) -> ResultOpenHems<bool> {
		let data = json!({
			"message": message,
			"title": "Notification from OpenHEMS."
		});
		self.call_api(
			"/services/notify/persistent_notification", Some(data)
		)?;
		Ok(true)
	}
	fn get_cycle_id(&self) -> u32 {
		self.cycle_id
	}
	fn register_entity(&mut self, nameid:&str) -> bool {
		println!("register_entity({})", nameid);
		if !self.cached_ids.contains_key(nameid) {
			self.cached_ids.insert(nameid.to_string(), JsonValue::Null);
		}
		true
	}
    get_entity_value_!(get_entity_value_int, i32, as_i32);
    get_entity_value_!(get_entity_value_float, f32, as_f32);
	fn get_entity_value_str(&self, entity_id:&str) -> ResultOpenHems<String> {
		let v= self.get_entity_value(entity_id)?;
		if let Some(value)  = v.as_str() {
			Ok(value.to_string())
		} else {
			let message = format!("Value can not be parsed as string : {:?}", v);
			Err(OpenHemsError::new(message))
		}
	}
	fn get_entity_value_bool(&self, entity_id:&str) -> ResultOpenHems<bool> {
		let v= self.get_entity_value(entity_id)?;
		match v {
			JsonValue::Boolean(value) => {
				Ok(*value)
			}
			JsonValue::Short(value) => {
				let v2 = value.as_str().to_lowercase();
				let false_value = ["0", "false", "off"].iter().any(|&s| s==v2);
				// log::debug!("HomeAssistantAPI::get_entity_value_bool({v2}) => not {false_value}");
				Ok(!false_value)
			}
			_ => {
				let message = format!("Value can not be parsed as bool : {:?} ", v);
				Err(OpenHemsError::new(message))
			}
		}
	}
}

#[derive(Clone)]
pub struct FakeNetworkUpdater {
	none: u32,
}

impl HomeStateUpdater for FakeNetworkUpdater {
    fn default() -> Self {
		FakeNetworkUpdater {none:0}
	}
    fn init_network(&mut self)-> ResultOpenHems<bool> {
		Ok(true)
	}
    fn update_network(&mut self) -> ResultOpenHems<bool> {
		Ok(true)
	}
	fn register_entity(&mut self, _nameid:&str) -> bool {
		true
	}
	fn get_entity_value_int(&self, _nameid:&str) -> ResultOpenHems<i32> {
		Ok(0)
	}
	fn get_entity_value_float(&self, _nameid:&str) -> ResultOpenHems<f32> {
		Ok(0.0)
	}
	fn get_entity_value_str(&self, _nameid:&str) -> ResultOpenHems<String> {
		Ok(String::from(""))
	}
	fn get_entity_value_bool(&self, _nameid:&str) -> ResultOpenHems<bool> {
		Ok(true)
	}
    // fn get_network(&self) -> Network;
    fn get_cycle_id(&self) -> u32 {
		0
	}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_test() -> Result<(), OpenHemsError> {
		let mut api = HomeAssistantAPI::default();
		api.init(
			String::from("http://192.168.1.202:8123/api"),
			String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJmOTM4ZTFmY2FjNTA0MWEyYWZkYjEyOGYyYTJlNGNmYiIsImlhdCI6MTcyNjU2NTU1NiwiZXhwIjoyMDQxOTI1NTU2fQ.3DdEXGsM3cg5NgMUKj2k5FsEG07p2AkRF_Ad-CljSTQ")
		)?;
		let states = api.update_network()?;
		assert_eq!(states, true);
		println!("{states:?}");
	
		let states = api.notify("Hello world!")?;
		assert_eq!(states, true);
		println!("{states:?}");
		Ok(())
    }
}