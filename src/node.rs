
use core::fmt;

pub trait Node {
    // Attributes
	fn get_id(&self) -> &str;
    fn get_min_power(&self) -> f32;
    fn get_max_power(&self) -> f32;
    fn get_current_power(&self) -> f32;
    fn is_on(&self) -> bool;
    fn is_activate(&self) -> bool;
    // TODO : fn get_constraints(&self) -> bool;
}

impl fmt::Display for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Node({}, MaxPower:{})", self.get_id(), self.get_max_power())
    }
}
impl fmt::Debug for dyn Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "Node({}, MaxPower:{})", self.get_id(), self.get_max_power())
    }
}

pub struct Switch {
	// Node
	nameid: String,
	// Outnode
	// Switch
	pritority: u32,
	strategy_nameid: String,
	schedule: u32
}
pub fn get_switch(nameid: String, pritority: u32, strategy_nameid: String) -> Switch {
	Switch {
		nameid: nameid,
		pritority: pritority,
		strategy_nameid: strategy_nameid,
		schedule: 0
	}
}
impl Node for Switch {
    // Attributes
	fn get_id(&self) -> &str {
		&self.nameid
	}
    fn get_min_power(&self) -> f32 {
		0.0
	}
    fn get_max_power(&self) -> f32 {
		0.0
	}
    fn get_current_power(&self) -> f32 {
		0.0
	}
    fn is_on(&self) -> bool {
		true
	}
    fn is_activate(&self) -> bool {
		true
	}
}