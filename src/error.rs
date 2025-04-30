use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct OpenHemsError {
	message: String,
	// level: log::Level
}
impl fmt::Display for OpenHemsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::Error for OpenHemsError {

}
impl OpenHemsError {
	pub fn new(message:String) -> OpenHemsError {
		OpenHemsError {
			message: message,
			// level:log::Level::Debug
		}
	}
}
pub type ResultOpenHems<T> = std::result::Result<T, OpenHemsError>;
