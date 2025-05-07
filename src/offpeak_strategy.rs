use datetime::{LocalTime, LocalDateTime, LocalDate, Month};
use crate::error::ResultOpenHems;
use crate::network::Network;
use crate::node::Node;
use crate::home_assistant_api::HomeStateUpdater;
use crate::time::{HoursRanges, HoursRange};

pub trait EnergyStrategyk<'a, T:HomeStateUpdater> {
	fn get_strategy_id(&self) -> &str;
	fn get_network(&self) -> &Network<'a, T>;
	fn get_nodes(&self) -> &Vec<Box<dyn Node>>;
	fn update_network(&self) -> ResultOpenHems<()>;
}
#[derive(Clone)]
pub struct OffPeakStrategy<'a, T:HomeStateUpdater> {
	id: String,
	inoffpeakrange: bool,
	rangechangedone: bool,
	currrange: &'a HoursRange,
	hoursranges: &'a HoursRanges,
	nextranges: Vec<HoursRange>,
	network: &'a Network<'a, T>
}

impl<'a, T:HomeStateUpdater> EnergyStrategyk<'a, T> for OffPeakStrategy<'a, T> {
	fn get_strategy_id(&self) -> &str {
		&self.id
	}
	fn get_network(&self) -> &Network<'a, T> {
		self.network
	}
	fn get_nodes(&self) -> &Vec<Box<dyn Node>> {
		todo!();
	}
	fn update_network(&self) -> ResultOpenHems<()>{
		todo!();
		Ok(())
	}
}
	
impl<'a, T:HomeStateUpdater> OffPeakStrategy<'a, T> {
	pub fn new(network:&'a Network<'a, T>, id:&str) -> ResultOpenHems<OffPeakStrategy<'a, T>>{
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