mod home_assistant_api;
mod configuration_manager;

fn main() {
    println!("Hello, world!");
	let file_path = "./config/openhems.yaml";
	let mut configurator = configuration_manager::get();
	let _ = configurator.add_yaml_config(file_path);
}
