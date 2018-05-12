extern crate iron;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::path::{Path,PathBuf};
use self::iron::prelude::*;
use self::iron::Handler;
use self::iron::method::Method;
use std::sync::{Arc,Mutex};

use self::iron::headers::ContentType;
use self::iron::mime::{Mime, TopLevel, SubLevel, Attr, Value};

pub struct ServerOptions {
	pub listen_addr: String,
}

pub fn http_server_start(opts: ServerOptions) -> bool {
	let dispatch = DispatchHandler{
		api_handler: APIHandler {},
	};

	Iron::new(dispatch).http(&*opts.listen_addr).unwrap();
	return true;
}

struct DispatchHandler	{
	api_handler: APIHandler,
}

impl Handler for DispatchHandler {

	fn handle(&self, req: &mut Request) -> IronResult<Response> {
		// forward /api/v1 to APIHandler
		if req.url.path().len() >= 2 &&
			 req.url.path()[0] == "api" &&
			 req.url.path()[1] == "v1" {
			return self.api_handler.handle(req);
		}

		// return 200 for /ping
		if req.url.path().len() == 1 && req.url.path()[0] == "ping" {
			let res = Response::with(iron::status::Ok);
			return Ok(res);
		}

		// return 404 for invalid routes
		return Ok(Response::with(iron::status::NotFound))
	}

}

struct APIHandler {}

impl APIHandler {

	fn handle(&self, req: &mut Request) -> IronResult<Response> {
		let invalid_request = Response::with((
				iron::status::BadRequest,
				"{ \"error\": \"invalid API request\" }"));

		if req.method != Method::Post ||
			 req.url.path().len() < 3 ||
			 req.url.path()[0] != "api" ||
			 req.url.path()[1] != "v1" {
			return Ok(invalid_request);
		}

		let method = req.url.path()[2].to_string();
		let mut body = Vec::<u8>::new();
		let body_str = match req.body.read_to_end(&mut body) {
			Ok(_) => String::from_utf8_lossy(&body),
			Err(_) => return Ok(invalid_request)
		};

		let mut res = match ::api_json::call_str(&method, &body_str) {
			Ok(res) =>
				Response::with((iron::status::Ok, res)),
			Err(err) => match err.code {
				::ErrorCode::BadRequest =>
						Response::with((iron::status::BadRequest, err.message)),
				::ErrorCode::NotFound =>
						Response::with((iron::status::NotFound, err.message)),
				::ErrorCode::InternalServerError =>
						Response::with((iron::status::InternalServerError, err.message)),
			}
		};

		res.headers.set(
				ContentType(
						Mime(
								TopLevel::Application,
								SubLevel::Json,
								vec![(Attr::Charset, Value::Utf8)])));

		return Ok(res);
	}

}

