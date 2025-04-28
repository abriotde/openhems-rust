mod home_assistant_api;

fn main() {
    println!("Hello, world!");
	let mut api = home_assistant_api::get_home_assistant_api(
		String::from("http://192.168.1.202:8123/api"),
		String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJmOTM4ZTFmY2FjNTA0MWEyYWZkYjEyOGYyYTJlNGNmYiIsImlhdCI6MTcyNjU2NTU1NiwiZXhwIjoyMDQxOTI1NTU2fQ.3DdEXGsM3cg5NgMUKj2k5FsEG07p2AkRF_Ad-CljSTQ")
	);
	let states = api.update_network();
	println!("{states:?}")
}
