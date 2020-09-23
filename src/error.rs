/**
 * Copyright © 2018 nyantec GmbH <oss@nyantec.com>
 * Authors:
 *   Paul Asmuth <asm@nyantec.com>
 *   Karl Engelhardt <ken@nyantec.com>
 *
 * Provided that these terms and disclaimer and all copyright notices
 * are retained or reproduced in an accompanying document, permission
 * is granted to deal in this work without restriction, including un‐
 * limited rights to use, publicly perform, distribute, sell, modify,
 * merge, give away, or sublicence.
 *
 * This work is provided “AS IS” and WITHOUT WARRANTY of any kind, to
 * the utmost extent permitted by applicable law, neither express nor
 * implied; without malicious intent or gross negligence. In no event
 * may a licensor, author or contributor be held liable for indirect,
 * direct, other damage, loss, or other issues arising in any way out
 * of dealing in the work, even if advised of the possibility of such
 * damage or existence of a defect, except proven that it results out
 * of said person’s immediate fault when using the work as intended.
 */
use std;

#[derive(Debug)]
pub enum ErrorCode {
	InternalServerError,
	BadRequest,
	QuotaError,
}

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

#[allow(unused_macros)]
macro_rules! err_quota {
	($($arg:tt)*) => (::Error::new(&format!($($arg)*), ::ErrorCode::QuotaError))
}

#[allow(unused_macros)]
macro_rules! fatal {
	($($arg:tt)*) => ({
		error!("FATAL ERROR: {}; aborting...", &format!($($arg)*));
		::std::process::abort();
	})
}

impl Error {
	pub fn new(message: &str, code: ErrorCode) -> Error {
		Error {
			message: message.to_owned(),
			code,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		return write!(f, "ERROR ({:?}): {}", self.code, self.message);
	}
}

impl std::convert::From<()> for Error {
	fn from(_: ()) -> Error {
		return err_server!("error: unknown error");
	}
}

impl std::convert::From<std::num::ParseIntError> for Error {
	fn from(e: std::num::ParseIntError) -> Error {
		return err_server!("Invalid integer: {:?}", e);
	}
}

impl std::convert::From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Error {
		return err_server!("I/O error: {:?}", e);
	}
}

impl std::convert::From<std::ffi::OsString> for Error {
	fn from(e: std::ffi::OsString) -> Error {
		return err_server!("FFI error: {:?}", e);
	}
}
