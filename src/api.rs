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
use ::logfile_service::LogfileService;

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
		logfile_service: &LogfileService,
		req: StoreMeasurementRequest) -> Result<StoreMeasurementResponse, ::Error> {
	debug!("Storing measurement: sensor_id={}", req.sensor_id);

	let logfile = logfile_service.logfile_map.lookup_or_create(&req.sensor_id)?;
	logfile.append_measurement(&req.time, req.data.as_bytes())?;

	return Ok(StoreMeasurementResponse {
		success: true
	});
}

