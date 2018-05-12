use std;

#[derive(Debug)]
pub enum ErrorCode { InternalServerError, NotFound, BadRequest }

#[derive(Debug)]
pub struct Error {
	pub message: String,
	pub code: ErrorCode,
}

#[allow(unused_macros)]
macro_rules! err_server {
	($($arg:tt)*) => (::Error::new(&format!($($arg)*), ::ErrorCode::InternalServerError))
}

#[allow(unused_macros)]
macro_rules! err_user {
	($($arg:tt)*) => (::Error::new(&format!($($arg)*), ::ErrorCode::BadRequest))
}

impl Error {

	pub fn new(message: &str, code: ErrorCode) -> Error {
		return Error {
			message: message.to_owned(),
			code: code
		};
	}

}

impl std::fmt::Display for Error {

	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		return write!(f, "ERROR ({:?}): {}", self.code, self.message);
	}

}

impl std::convert::From<()> for Error {

	fn from(e: ()) -> Error {
		return err_server!("error: unknown error");
	}

}

impl std::convert::From<std::io::Error> for Error {

	fn from(e: std::io::Error) -> Error {
		return err_server!("I/O error: {:?}", e);
	}

}

