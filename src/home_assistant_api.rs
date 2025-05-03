use std::collections::HashMap;
use reqwest;
use json::{self, JsonValue, object::Object};
use yaml_rust2::Yaml;
use core::fmt;
use std::io::Read;
use serde_json::json;
use crate::{
	cast_utility, error::ResultOpenHems, network::NodesHeap, node::{self, Node, NodeBase, PublicPowerGrid, Switch}
};

pub trait HomeStateUpdater {
    fn notify(&self, message:&str) -> Result<bool, reqwest::Error> {
		print!("HomeStateUpdater.notify : {message}");
		Ok(true)
	}
    fn init_network(&mut self);
    fn update_network(&mut self) -> Result<bool, reqwest::Error>;
    // fn switch_on(&self) -> bool;
    // fn get_feeder(&self, value:&str, expectedType:&str);
    fn get_publicpowergrid(&self,nameid:&str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<PublicPowerGrid>;
    // fn get_solarpanel(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
    // fn get_battery(&self,nameid:&str, nodeConf:HashMap<String, Yaml>);
    fn get_switch(&self,nameid:&str, node_conf:HashMap<String, &Yaml>)  -> ResultOpenHems<Switch>;
    // fn get_network(&self) -> Network;
    fn get_cycle_id(&self) -> i64 {
		// self.network.get_cycle_id()
		0
    }
}

#[derive(Debug)]
pub struct HomeAssistantAPI {
    token: String,
    url: String,
    network: u64,
    cached_ids: HashMap<String, String>,
	ha_elements: HashMap<String, Object>
}
pub fn get(ha_url: String, long_lived_token: String) -> HomeAssistantAPI {
	HomeAssistantAPI {
		token: long_lived_token,
		url: ha_url,
		network: 0,
		cached_ids: HashMap::new(),
		ha_elements: HashMap::new()
	}
}
impl fmt::Display for HomeAssistantAPI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "HomeAssistantAPI()")
    }
}

impl HomeAssistantAPI {
	// TODO : post with data + Status...
	pub fn call_api(&self, url: &str, data: Option<serde_json::Value>) -> Result<String, reqwest::Error> {
		// let client = reqwest::Client::new();
		let client = reqwest::blocking::Client::new();
		let mut bearer = String::from("Bearer ");
		bearer.push_str(&self.token);
		let mut complete_url: String = String::from(&self.url);
		complete_url.push_str(url);

		let request_builder;
		if let Some(mydata) = data {
			// let mut body = reqwest::blocking::Body::new(mydata);
			let request_body_string = serde_json::to_string(&mydata).unwrap();
			request_builder = client.post(&complete_url)
				.body(request_body_string);
		} else {
			request_builder = client.get(&complete_url);
		}
		let res = request_builder
			.header("Authorization", bearer)
			.header("content-type", "application/json")
			.send();
		match res {
			Ok(mut a) => {
				let mut body = String::new();
				let _ = a.read_to_string(&mut body);
				println!("Status: {}", a.status());
				println!("Headers:\n{:#?}", a.headers());
				// println!("Body:\n{}", body);
				println!("Call API !");
				Ok(body)
			}
			Err(b) => {
				println!("Unable to access Home Assistance instance, check URL : {complete_url:?} : {b:?}");
				Err(b)
			}
		}
	}
	fn get_feeder_int(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:i32) -> i32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_int(val)
		} else {
			default_value
		}
	}
	fn get_feeder_str(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:&str) -> String {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_str(val)
		} else {
			default_value.to_string()
		}
	}
	fn get_feeder_float(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:f32) -> f32 {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_float(val)
		} else {
			default_value
		}
	}
	fn get_feeder_bool(&self, node_conf:&HashMap<String, &Yaml>, key:&str, default_value:bool) -> bool {
		if let Some(val) = node_conf.get(key) {
			cast_utility::to_type_bool(val)
		} else {
			default_value
		}
	}
	fn get_nodebase<'a>(&self,nameid:&'a str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<NodeBase> {
		let max_power = self.get_feeder_float(&node_conf, "max_power", 0.0);
		let min_power = self.get_feeder_float(&node_conf, "min_power", 0.0);
		let current_power = self.get_feeder_float(&node_conf, "current_power", 0.0);
		let is_on = self.get_feeder_bool(&node_conf, "is_on", false);
		let node = node::get_nodebase(nameid, max_power, min_power, current_power, is_on)?;
		Ok(node)
	}
}


impl HomeStateUpdater for HomeAssistantAPI {
    fn init_network(&mut self) {

	}
	fn update_network(&mut self) -> Result<bool, reqwest::Error> {
		let states = self.call_api("/states", None)?;
		let mut count = 0;
		if let JsonValue::Array(parsed_list) = json::parse(&states).unwrap() {
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
	fn notify(&self, message: &str) -> Result<bool, reqwest::Error> {
		let data = json!({
			"message": message,
			"title": "Notification from OpenHEMS."
		});
		self.call_api(
			"/services/notify/persistent_notification", Some(data)
		)?;
		Ok(true)
	}
	fn get_switch<'a>(&self,nameid:&'a str, node_conf:HashMap<String, &Yaml>) -> ResultOpenHems<Switch> {
		// println!("HA:get_switch({nameid})");
		let priority = self.get_feeder_int(&node_conf, "priority", 50);
		let strategy_nameid = self.get_feeder_str(&node_conf, "strategy", "default");
		let base = self.get_nodebase(nameid, node_conf)?;
		let node = node::get_switch(base, priority as u32, &strategy_nameid)?;
		Ok(node)
	}
	fn get_publicpowergrid<'a>(&'a self,nameid:&str, node_conf:HashMap<String, &Yaml>)  -> ResultOpenHems<PublicPowerGrid> {
		let base = self.get_nodebase(nameid, node_conf)?;
		node::get_publicpowergrid(base, 0)
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