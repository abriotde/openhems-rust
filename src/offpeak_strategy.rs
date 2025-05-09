use datetime::LocalDateTime;
use hashlink::linked_hash_map::LinkedHashMap;
use crate::error::ResultOpenHems;
use crate::network::Network;
use crate::node::Node;
use crate::time::{HoursRanges, HoursRange};
use yaml_rust2::Yaml;

pub trait EnergyStrategy<'a, 'b:'a, 'c:'b> {
	fn get_strategy_id(&self) -> &str;
	fn get_network(&self) -> &Network<'a, 'a>;
	fn get_nodes(&self) -> &Vec<Box<dyn Node>>;
	fn update_network(&self) -> ResultOpenHems<u32>;
	fn new(network:&'c Network<'a, 'a>, id:&str, config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy<'a, 'a>>;
}
#[derive(Clone)]
pub struct OffPeakStrategy<'a, 'b:'a> {
	id: String,
	inoffpeakrange: bool,
	rangechangedone: bool,
	currrange: &'a HoursRange,
	hoursranges: &'a HoursRanges,
	nextranges: Vec<HoursRange>,
	network: &'b Network<'a, 'a>
}

impl<'a, 'b:'a, 'c:'b, 'd:'c> EnergyStrategy<'a, 'b, 'c> for OffPeakStrategy<'a, 'a> {
	fn get_strategy_id(&self) -> &str {
		&self.id
	}
	fn get_network(&self) -> &Network<'a, 'a> {
		self.network
	}
	fn get_nodes(&self) -> &Vec<Box<dyn Node>> {
		todo!();
	}
	fn update_network(&self) -> ResultOpenHems<u32>{
		todo!();
		// Ok(0)
	}
	fn new(network:&'b Network<'a, 'a>, id:&str, _config:&LinkedHashMap<Yaml, Yaml>) -> ResultOpenHems<OffPeakStrategy<'a, 'a>> {
		let hoursranges = network.get_hours_ranges()?;
		let now = LocalDateTime::now();
		let range = hoursranges.check_range(now)?;
		Ok(OffPeakStrategy {
			id: id.to_string(),
			inoffpeakrange: hoursranges.is_offpeak(range),
			rangechangedone: false,
			currrange: range,
			hoursranges: hoursranges,
			nextranges: Vec::new(),
			network: network
		})
	}
}

impl<'a, 'b:'a, 'c:'b, 'd:'c> OffPeakStrategy<'a, 'a> {
	pub fn get_id(&self) -> &str {
		&self.id
	}
}
