
use hashlink::linked_hash_map::LinkedHashMap;
use yaml_rust2::Yaml;

pub fn get_yaml_key<'a>(key:&str, config:&'a LinkedHashMap<Yaml, Yaml>) -> Option<&'a Yaml> {
	let k = Yaml::String(key.to_string());
	if let Some(value) = config.get(&k) {
		Some(value)
	} else {
		None
	}
}