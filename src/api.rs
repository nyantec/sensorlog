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
use ::service::Service;
use ::logfile_id::LogfileID;
use ::measure::Measurement;

#[derive(Serialize, Deserialize)]
pub struct StoreMeasurementRequest {
	time: Option<u64>,
	sensor_id: String,
	data: String,
}

#[derive(Serialize, Deserialize)]
pub struct StoreMeasurementResponse {
	success: bool
}

pub fn store_measurement(
		service: &Service,
		req: StoreMeasurementRequest) -> Result<StoreMeasurementResponse, ::Error> {
	debug!("Storing measurement: sensor_id={}", req.sensor_id);

	let measurement = Measurement {
		time: req.time.unwrap_or(::time::get_unix_microseconds()?),
		data: req.data.as_bytes().to_vec(),
	};

	let logfile_id = LogfileID::from_string(req.sensor_id.to_owned());
	let logfile = service.logfile_map.lookup_or_create(&logfile_id)?;
	logfile.append_measurement(&measurement)?;

	return Ok(StoreMeasurementResponse {
		success: true
	});
}

#[derive(Serialize, Deserialize)]
pub struct FetchLastMeasurementRequest {
	sensor_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct FetchLastMeasurementResponse {
	measurement: Option<Measurement>,
}

pub fn fetch_last_measurement(
		service: &Service,
		req: FetchLastMeasurementRequest) -> Result<FetchLastMeasurementResponse, ::Error> {
	let logfile_id = LogfileID::from_string(req.sensor_id.to_owned());

	debug!("Fetching last measurement: sensor_id={}", req.sensor_id);
	let measurement = match service.logfile_map.lookup(&logfile_id) {
		Some(logfile) => logfile.fetch_last_measurement()?,
		None => None
	};

	return Ok(FetchLastMeasurementResponse{
		measurement: measurement
	});
}

