use serde::{Serialize, Deserialize};
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
	return Err(err_server!("not implemented"));
}

