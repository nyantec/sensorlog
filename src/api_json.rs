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
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json as json;
use ::service::Service;

pub fn call_str(
		service: &Service,
		method: &str,
		req: &str) -> Result<String, ::Error> {
	let req_json = match json::from_str(&req) {
		Ok(json) => json,
		Err(e) => return Err(err_user!("invalid JSON: {}", e))
	};

	let res = call_json(service, method, &req_json)?;
	let res_json = match json::to_string(&res) {
		Ok(json) => json,
		Err(e) => return Err(err_server!("error while encoding JSON: {}", e))
	};

	return Ok(res_json);
}

pub fn call_json(
		service: &Service,
		method: &str,
		req: &json::Value) -> Result<json::Value, ::Error> {
	debug!("Executing API request: method={}", method);

	return match method {
		"store_measurement" => call(service, &::api::store_measurement, req),
		"fetch_measurements" => call(service, &::api::fetch_measurements, req),
		_ => Err(err_user!("invalid API method"))
	};
}

fn call<RequestType: DeserializeOwned, ResponseType: Serialize>(
		service: &Service,
		method: &Fn(&Service, RequestType) -> Result<ResponseType, ::Error>,
		params: &json::Value) -> Result<json::Value, ::Error> {
	let req : RequestType = match json::from_value(params.to_owned()) {
		Ok(r) => r,
		Err(e) => return Err(err_user!("invalid request: {}", e))
	};

	return match method(service, req) {
		Ok(r) => Ok(json::to_value(r).unwrap()),
		Err(e) => Err(e)
	};
}

