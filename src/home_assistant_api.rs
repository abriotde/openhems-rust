use std::{collections::HashMap, fmt::format};
use reqwest;
use json::{self, JsonValue, object::Object};
use yaml_rust2::Yaml;
use core::fmt;
use std::io::Read;
use serde_json::json;
use crate::{
	cast_utility, error::{OpenHemsError, ResultOpenHems}, node::{self, NodeBase, PublicPowerGrid, Switch},
	feeder::SourceFeeder
};

pub trait HomeStateUpdater 
	where Self:Clone
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

	// fn switch_on(&self) -> bool;
	// fn get_feeder(&self, value:&str, expectedType:&str);
	fn get_publicpowergrid(&self,nameid:&str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<PublicPowerGrid<Self>>;
	// fn get_solarpanel(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
	// fn get_battery(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
	fn get_switch(&self, nameid:&str, node_conf:HashMap<String, &Yaml>)  -> ResultOpenHems<Switch<Self>>;
    // fn get_network(&self) -> Network;
    fn get_cycle_id(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct HomeAssistantAPI {
    token: String,
    url: String,
    network: u64,
    cached_ids: HashMap<String, JsonValue>,
	ha_elements: HashMap<String, Object>,
	cycle_id:u32
}
impl fmt::Display for HomeAssistantAPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "HomeAssistantAPI()")
    }
}

impl<'a> HomeAssistantAPI {
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
	fn get_feeder_const_int(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:i32) -> i32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_int(val)
		} else {
			default_value
		}
	}
	fn get_feeder_const_str(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:&str) -> String {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_str(val)
		} else {
			default_value.to_string()
		}
	}
	fn get_feeder_const_float(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:f32) -> f32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_float(val)
		} else {
			default_value
		}
	}
	fn get_feeder_float(&'a self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:f32) -> ResultOpenHems<SourceFeeder<'a, HomeAssistantAPI, f32>> {
		if let Some(val) = node_conf.get(key) {
			if let Yaml::String(entity_id) = val {
				if self.ha_elements.contains_key(entity_id) {
					// <HomeAssistantAPI, f32>
					SourceFeeder::new(self, entity_id)
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
	fn get_feeder_bool(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:bool) -> bool {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_bool(val)
		} else {
			default_value
		}
	}
	fn get_nodebase(&self,nameid:&'a str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<NodeBase<HomeAssistantAPI>> {
		let max_power = self.get_feeder_const_float(&node_conf, "maxPower", 0.0);
		let min_power = self.get_feeder_const_float(&node_conf, "minPower", 0.0);
		let current_power = self.get_feeder_float(&node_conf, "currentPower", 0.0)?;
		let is_on = self.get_feeder_bool(&node_conf, "is_on", false);
		let node = node::get_nodebase(nameid, max_power, min_power, current_power, is_on)?;
		Ok(node)
	}
	fn get_entity_value(&self, entity_id:&str) -> ResultOpenHems<&JsonValue> {
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
}


impl HomeStateUpdater for HomeAssistantAPI {
    fn default() -> Self {
		HomeAssistantAPI {
			token: "".to_string(),
			url: "".to_string(),
			network: 0,
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
					self.ha_elements.insert(String::from(entity_id), entity);
					count = count + 1;
				}
			}
		}
		println!("Count {count}");
		Ok(true)
	}
	fn update_network(&mut self) -> ResultOpenHems<bool> {
		self.cycle_id += 1;
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
	fn get_switch(&self, nameid:&str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<Switch<HomeAssistantAPI>> {
		// println!("HA:get_switch({nameid})");
		let priority = self.get_feeder_const_int(&node_conf, "priority", 50);
		let strategy_nameid = self.get_feeder_const_str(&node_conf, "strategy", "default");
		let base = self.get_nodebase(nameid, node_conf)?;
		node::get_switch(base, priority as u32, &strategy_nameid)
	}
	fn get_publicpowergrid<'a>(&'a self, nameid:&str, node_conf:HashMap<String, &Yaml>)  -> ResultOpenHems<PublicPowerGrid<'a, HomeAssistantAPI>> {
		let base: NodeBase<'_, HomeAssistantAPI> = self.get_nodebase(nameid, node_conf)?;
		node::get_publicpowergrid(base, 0)
	}
	fn register_entity(&mut self, nameid:&str) -> bool {
		if !self.cached_ids.contains_key(nameid) {
			self.cached_ids.insert(nameid.to_string(), JsonValue::Null);
		}
		true
	}
	fn get_entity_value_int(&self, entity_id:&str) -> ResultOpenHems<i32> {
		let v= self.get_entity_value(entity_id)?;
		if let Some(value)  = v.as_i32() {
			Ok(value)
		} else {
			let message = format!("Value can not be parsed as Integer.");
			Err(OpenHemsError::new(message))
		}
	}
	fn get_entity_value_float(&self, entity_id:&str) -> ResultOpenHems<f32> {
		let v= self.get_entity_value(entity_id)?;
		if let Some(value)  = v.as_f32() {
			Ok(value)
		} else {
			let message = format!("Value can not be parsed as Real.");
			Err(OpenHemsError::new(message))
		}
	}
	fn get_entity_value_str(&self, entity_id:&str) -> ResultOpenHems<String> {
		let v= self.get_entity_value(entity_id)?;
		if let Some(value)  = v.as_str() {
			Ok(value.to_string())
		} else {
			let message = format!("Value can not be parsed as string");
			Err(OpenHemsError::new(message))
		}
	}
}



#[cfg(test)]
mod tests {
    use reqwest::Error;

    use super::*;

    #[test]
    fn local_test() -> Result<(), Error> {
		let mut api = get(
			String::from("http://192.168.1.202:8123/api"),
			String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJmOTM4ZTFmY2FjNTA0MWEyYWZkYjEyOGYyYTJlNGNmYiIsImlhdCI6MTcyNjU2NTU1NiwiZXhwIjoyMDQxOTI1NTU2fQ.3DdEXGsM3cg5NgMUKj2k5FsEG07p2AkRF_Ad-CljSTQ")
		);
		let states = api.update_network()?;
		assert_eq!(states, true);
		println!("{states:?}");
	
		let states = api.notify("Hello world!")?;
		assert_eq!(states, true);
		println!("{states:?}");
		Ok(())
    }
}