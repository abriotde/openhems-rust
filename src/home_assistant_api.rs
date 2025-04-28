use std::collections::HashMap;
use reqwest;
use json::{self, JsonValue, object::Object};
// use error_chain::error_chain;
use std::io::Read;
use serde_json::json;

/* error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
} */

#[derive(Debug)]
pub struct HomeAssistantAPI {
    token: String,
    url: String,
    network: u64,
    cached_ids: HashMap<String, String>,
	ha_elements: HashMap<String, Object>
}
pub fn get_home_assistant_api(ha_url: String, long_lived_token: String) -> HomeAssistantAPI {
	HomeAssistantAPI {
		token: long_lived_token,
		url: ha_url,
		network: 0,
		cached_ids: HashMap::new(),
		ha_elements: HashMap::new()
	}
}

impl HomeAssistantAPI {
	pub fn update_network (&mut self) -> Result<bool, reqwest::Error> {

		let states = self.call_api("/states", None)?;
		let mut count = 0;
		if let JsonValue::Array(parsedList) = json::parse(&states).unwrap() {
			for elem in parsedList {
				if let JsonValue::Object(entity) = elem {
					let entity_id = entity.get("entity_id").unwrap().as_str().unwrap();
					println!(" - {entity_id}");
					self.ha_elements.insert(String::from(entity_id), entity);
					count = count + 1;
				}
			}
		}
		println!("Count {count}");
		Ok(true)
	}
	pub fn notify(&self, message: &str) -> Result<bool, reqwest::Error> {
		let data = json!({
			"message": message,
			"title": "Notification from OpenHEMS."
		});
		self.call_api(
			"/services/notify/persistent_notification", Some(data)
		)?;
		Ok(true)
	}
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
}