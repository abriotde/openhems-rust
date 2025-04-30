use std::{collections::HashMap, fs};
use std::error::Error;
use yaml_rust2::{Yaml, YamlLoader};
use crate::cast_utility;

pub struct ConfigurationManager {
	conf:HashMap<String, Box<Yaml>>,
	cache:HashMap<String, String>,
	default_path:String
}

pub fn get(default_path:Option<String>) -> ConfigurationManager {
	let mut conf = ConfigurationManager {
		conf: HashMap::new(),
		cache: HashMap::new(),
		default_path: String::from("")
	};
	if let Some(path) = default_path {
		let _ = conf.add_yaml_config(&path, true);
	} else {
		let path = "./data/openhems_default.yaml";
		let _ = conf.add_yaml_config(path, true);
	}
	conf
}

impl ConfigurationManager {
	fn add(&mut self, key:&str, value:&Yaml, init:bool) {
		if let Yaml::Hash(config) = value {
			for (k, value) in config.into_iter() {
				let k_str = cast_utility::to_type_str(k);
				let mut newkey:String;
				if key!="" {
					newkey = String::from(key);
					newkey.push_str(".");
					newkey.push_str(&k_str);
				} else {
					newkey = k_str;
				}
				self.add(&newkey, value, init);
			}
		} else {
			let val = Box::new(value.clone());
			if !init && !self.conf.contains_key(key){
				println!("ERROR : key='{key}' is not valid in configuration.");
			} else {
				// println!(" - {key}");
				self.conf.insert(key.to_string(),val);
			}
		}
	}

	pub fn add_yaml_config(&mut self, file_path:&str, init:bool) -> Result<(), Box<dyn Error>> {
		self.default_path = String::from(file_path);
		let yaml_config: String = fs::read_to_string(file_path)?;
		// println!("Yaml configuration:{yaml_config}");
		let docs = YamlLoader::load_from_str(&yaml_config)?;
		let doc = &docs[0];
		self.add("", doc, init);
		Ok(())
	}
	pub fn get_as_str(&self, key:&str) -> String {
		if let Some(value) = self.conf.get(key) {
			cast_utility::to_type_str(value)
		} else {
			String::from("")
		}
	}
	pub fn get_as_int(&self, key:&str) -> i64 {
		if let Some(value) = self.conf.get(key) {
			cast_utility::to_type_int(value)
		} else {
			0
		}
	}
	pub fn get_as_float(&self, key:&str) -> f32 {
		if let Some(value) = self.conf.get(key) {
			cast_utility::to_type_float(value)
		} else {
			0.0
		}
	}
	pub fn get_as_list(&self, key:&str) -> Vec<&Yaml> {
		if let Some(value) = self.conf.get(key) {
			cast_utility::to_type_list(value)
		} else {
			Vec::new()
		}
	}
}