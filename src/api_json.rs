use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json as json;

pub fn call_str(method: &str, req: &str) -> Result<String, ::Error> {
	let req_json = match json::from_str(&req) {
		Ok(json) => json,
		Err(e) => return Err(err_user!("invalid JSON: {}", e))
	};

	let res = call_json(method, &req_json)?;
	let res_json = match json::to_string(&res) {
		Ok(json) => json,
		Err(e) => return Err(err_server!("error while encoding JSON: {}", e))
	};

	return Ok(res_json);
}

pub fn call_json(method: &str, req: &json::Value) -> Result<json::Value, ::Error> {
	return match method {
		"store_measurement" => call(&::api::store_measurement, req),
		_ => Err(err_user!("invalid API method"))
	};
}

fn call<RequestType: DeserializeOwned, ResponseType: Serialize>(
		method: &Fn(RequestType) -> Result<ResponseType, ::Error>,
		params: &json::Value) -> Result<json::Value, ::Error> {
	let req : RequestType = match json::from_value(params.to_owned()) {
		Ok(r) => r,
		Err(e) => return Err(err_user!("invalid request: {}", e))
	};

	return match method(req) {
		Ok(r) => Ok(json::to_value(r).unwrap()),
		Err(e) => Err(e)
	};
}

