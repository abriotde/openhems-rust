use yaml_rust2::Yaml;
use yaml_rust2::YamlLoader;
use crate::cast_utility;
use crate::{
	error::ResultOpenHems, time::HoursRanges
};

#[derive(Debug, Clone)]
pub struct Contract {
	timeslots: HoursRanges,
}
impl Contract {
	pub fn get_hoursranges(&self) -> &HoursRanges {
		&self.timeslots
	}
	pub fn get_from_conf(contract_conf: &Yaml) -> ResultOpenHems<Contract>{
		let default_config = "[\"22h-6h\"]";
		let configs = YamlLoader::load_from_str(default_config).unwrap();
		let mut config = &configs[0];
		let mut default_cost = 0.1;
		let mut outrange_cost = 1.0;
		if let Yaml::Hash(contract_conf)= contract_conf {
			let key = Yaml::String("class".to_string());
			if let Some(Yaml::String(classname)) = contract_conf.get(&key) {
				println!("Contract : {classname}");
			} else {
				log::error!("No key 'classname' in contract, use default : '{default_config}'.");
			}
			let key = Yaml::String("offpeakhoursranges".to_string());
			if let Some(c) = contract_conf.get(&key) {
				config = c;
			}
			let toupdate = [
				("outRangePrice", &mut outrange_cost),
				("defaultPrice", &mut default_cost)
			];
			for (k,v) in toupdate {
				let key = Yaml::String(k.to_string());
				if let Some(value) = contract_conf.get(&key) {
					*v = cast_utility::to_type_float(value);
				} else {
					log::info!("No key '{k}' in contract, use default : '{}'.", *v);
				}
			}
		}
		let ranges = HoursRanges::from(config, 
			None, None, None, 
			default_cost, outrange_cost)?;
		Ok(Contract {
			timeslots: ranges,
		})
	}
}