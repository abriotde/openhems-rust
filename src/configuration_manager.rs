use std::{collections::HashMap, fs};
use std::error::Error;
use yaml_rust2::{Yaml, YamlLoader};

pub fn cast_str(value: &yaml_rust2::Yaml) -> String {
	match value {
		Yaml::Real(v) => {
			v.to_string()
		}
		Yaml::Integer(v) => {
			v.to_string()
		}
		Yaml::String(v) => {
			v.to_string()
		}
		Yaml::Boolean(v) => {
			if *v {
				String::from("True")
			} else {
				String::from("False")
			}
		}
		Yaml::Array(_) => {
			String::from("")
		}
		Yaml::Hash(_) => {
			String::from("")
		}
		Yaml::Alias(_) => {
			String::from("")
		}
		Yaml::Null => {
			String::from("")
		}
		Yaml::BadValue => {
			String::from("")
		}
	}
}

pub struct ConfigurationManager {
	conf:HashMap<String, Box<Yaml>>,
	cache:HashMap<String, String>,
	default_path:String
}

pub fn get () -> ConfigurationManager {
	ConfigurationManager {
		conf: HashMap::new(),
		cache: HashMap::new(),
		default_path: String::from("")
	}
}

impl ConfigurationManager {

	fn add(&mut self, key:&str, value:&Yaml) {
		if let Yaml::Hash(config) = value {
			for (k, value) in config.into_iter() {
				let k_str = cast_str(k);
				let mut newkey:String;
				if key!="" {
					newkey = String::from(key);
					newkey.push_str(".");
					newkey.push_str(&k_str);
				} else {
					newkey = k_str;
				}
				self.add(&newkey, value);
			}
		} else {
			let val = Box::new(value.clone());
			println!(" - {key}");
			self.conf.insert(key.to_string(),val);
		}
	}

	pub fn add_yaml_config(&mut self, file_path:&str) -> Result<(), Box<dyn Error>> {
		self.default_path = String::from(file_path);
		let yaml_config: String = fs::read_to_string(file_path)?;
		// println!("Yaml configuration:{yaml_config}");
		let docs = YamlLoader::load_from_str(&yaml_config)?;
		let doc = &docs[0];
		self.add("", doc);
		Ok(())
	}
}