use yaml_rust2::Yaml;
use std::collections::HashMap;

pub fn to_type_str(value: &Yaml) -> String {
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
pub fn to_type_int(value: &Yaml) -> i32 {
	match value {
		Yaml::Real(v) => {
			let r = v.parse::<f64>().unwrap();
			r as i32
		}
		Yaml::Integer(v) => {
			*v as i32
		}
		Yaml::String(v) => {
			v.parse::<i32>().unwrap()
		}
		Yaml::Boolean(v) => {
			if *v {
				1
			} else {
				0
			}
		}
		Yaml::Array(_) => {
			0
		}
		Yaml::Hash(_) => {
			0
		}
		Yaml::Alias(_) => {
			0
		}
		Yaml::Null => {
			0
		}
		Yaml::BadValue => {
			0
		}
	}
}
pub fn to_type_float(value: &Yaml) -> f32 {
	match value {
		Yaml::Real(v) => {
			v.parse::<f32>().unwrap()
		}
		Yaml::Integer(v) => {
			*v as f32
		}
		Yaml::String(v) => {
			v.parse::<f32>().unwrap()
		}
		Yaml::Boolean(v) => {
			if *v {
				1.0
			} else {
				0.0
			}
		}
		Yaml::Array(_) => {
			0.0
		}
		Yaml::Hash(_) => {
			0.0
		}
		Yaml::Alias(_) => {
			0.0
		}
		Yaml::Null => {
			0.0
		}
		Yaml::BadValue => {
			0.0
		}
	}
}
pub fn to_type_bool(value: &Yaml) -> bool {
	match value {
		Yaml::Real(_) => {
			false
		}
		Yaml::Integer(_) => {
			false
		}
		Yaml::String(v) => {
			if v.to_lowercase()=="true" || v=="1" {
				true
			} else {
				false
			}
		}
		Yaml::Boolean(v) => {
			*v
		}
		Yaml::Array(_) => {
			false
		}
		Yaml::Hash(_) => {
			false
		}
		Yaml::Alias(_) => {
			false
		}
		Yaml::Null => {
			false
		}
		Yaml::BadValue => {
			false
		}
	}
}
pub fn to_type_list(value: &Yaml) -> Vec<&Yaml> {
	match value {
		Yaml::Real(_) => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
		Yaml::Integer(_) => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
		Yaml::String(_) => {
			let mut vec = Vec::new();
			vec.push(value);
			// Try parse string as JSON
			vec
		}
		Yaml::Boolean(_) => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
		Yaml::Array(v) => {
			let mut vec = Vec::new();
			for v1 in v {
				vec.push(v1);
			}
			vec
		}
		Yaml::Hash(hash) => {
			let mut vec = Vec::new();
			for (_, v) in hash {
				vec.push(v);
			}
			vec
		}
		Yaml::Alias(_) => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
		Yaml::Null => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
		Yaml::BadValue => {
			let mut vec = Vec::new();
			vec.push(value);
			vec
		}
	}
}
pub fn to_type_dict(value: &Yaml) -> HashMap<String, &Yaml> {
	match value {
		Yaml::Real(_) => {
			HashMap::new()
		}
		Yaml::Integer(_) => {
			HashMap::new()
		}
		Yaml::String(_) => {
			HashMap::new()
		}
		Yaml::Boolean(_) => {
			HashMap::new()
		}
		Yaml::Array(_) => {
			HashMap::new()
		}
		Yaml::Hash(hash) => {
			let mut ret = HashMap::new();
			for (k, value) in hash {
				let key = to_type_str(k);
				ret.insert(key, value);
			}
			ret
		}
		Yaml::Alias(_) => {
			HashMap::new()
		}
		Yaml::Null => {
			HashMap::new()
		}
		Yaml::BadValue => {
			HashMap::new()
		}
	}
}