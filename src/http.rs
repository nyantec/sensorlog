/**
 * Copyright © 2018 nyantec GmbH <oss@nyantec.com>
 * Authors:
 *   Paul Asmuth <asm@nyantec.com>
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
use std::net::SocketAddr;
use std::sync::Arc;
use futures;
use futures::future::Future;
use futures::Stream;
use futures_cpupool::CpuPool;
use hyper;
use hyper::{StatusCode, Method, Uri};
use hyper::server::{Http, Request, Response};
use ::service::Service;

pub struct ServerOptions {
	pub listen_addr: String,
}

pub fn start_server(
		service: Arc<::service::Service>,
		opts: ServerOptions) -> Result<(), ::Error> {
	let listen_addr = match opts.listen_addr.parse::<SocketAddr>() {
		Ok(addr) => addr,
		Err(e) => return Err(err_user!("invalid listen address: {}", e))
	};

	info!("Listening for HTTP connections on {}", &opts.listen_addr);
	let server_factory = Http::new().bind(
			&listen_addr,
			move || Ok(HTTPHandler::new(service.clone())));

	let server = match server_factory {
		Ok(server) => server,
		Err(e) => return Err(err_user!("HTTP server rror: {}", e))
	};

	if let Err(e) = server.run() {
		return Err(err_server!("HTTP server error: {}", e));
	}

	return Ok(());
}

fn serve(
		service: &Service,
		method: Method,
		uri: Uri,
		body: &Vec<u8>) -> Result<Response, ::Error> {
	debug!("HTTP request; method={}, uri={}", method, uri);

	// route /api/v1/*
	if uri.path().starts_with("/api/v1/") {
		return serve_api(service, method, uri, body);
	}

	// route /ping
	if uri.path() == "/ping" {
		return Ok(
				Response::new()
						.with_status(StatusCode::Ok)
						.with_body("pong"));
	}

	// return 404 for invalid routes
	return Ok(
			Response::new()
					.with_status(StatusCode::NotFound)
					.with_body("not found"));
}

fn serve_api(
		service: &Service,
		method: Method,
		uri: Uri,
		body: &Vec<u8>) -> Result<Response, ::Error> {
	if method != Method::Post ||
	   !uri.path().starts_with("/api/v1/") {
		return Err(err_user!("invalid request"))
	}

	let rpc_body = String::from_utf8_lossy(&body);
	let rpc_method = uri
			.path()
			.split("/")
			.collect::<Vec<&str>>()[3]
			.to_string();

	let rpc_response = ::api_json::call_str(&service, &rpc_method, &rpc_body)?;

	return Ok(
			Response::new()
					.with_status(StatusCode::Ok)
					.with_body(rpc_response));
}


pub struct HTTPHandler {
	service: Arc<Service>,
	thread_pool: CpuPool,
}

impl HTTPHandler {

	pub fn new(service: Arc<Service>) -> HTTPHandler {
		return HTTPHandler {
			thread_pool: CpuPool::new_num_cpus(),
			service: service
		};
	}

}

impl hyper::server::Service for HTTPHandler {

	type Request = Request;
	type Response = Response;
	type Error = ::hyper::Error;
	type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

	fn call(&self, req: Request) -> Self::Future {
		let service = self.service.clone();

		let res_future = self.thread_pool.spawn_fn(move || {
			let (method, uri, _version, _headers, body) = req.deconstruct();

			let body_chunks = match body.collect().wait() {
				Ok(v) => v,
				Err(e) => {
					error!("HTTP Error while reading request body: {}", e);

					return futures::future::ok(
							Response::new()
									.with_status(StatusCode::InternalServerError)
									.with_body("error while reading request body"))
				}
			};

			let body = body_chunks.iter().fold(vec![], |mut acc, chunk| {
				acc.extend_from_slice(chunk.as_ref());
				acc
			});

			let res : Response = match serve(&service, method, uri, &body) {
				Ok(res) => res,
				Err(err) => {
					error!("HTTP Error: {}", err);

					match err.code {
						::ErrorCode::BadRequest =>
								Response::new()
										.with_status(StatusCode::BadRequest)
										.with_body(err.to_string()),
						::ErrorCode::InternalServerError =>
								Response::new()
										.with_status(StatusCode::InternalServerError)
										.with_body(err.to_string()),
						::ErrorCode::QuotaError =>
								Response::new()
										.with_status(StatusCode::Forbidden)
										.with_body(err.to_string()),
					}
				}
			};

			return futures::future::ok(res);
		});

		return Box::new(res_future);
	}

}

