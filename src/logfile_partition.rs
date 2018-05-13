/**
 * Copyright © 2018 nyantec GmbH <oss@nyantec.com>
 * Authors:
 *	 Paul Asmuth <asm@nyantec.com>
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
use ::measure::Measurement;

pub struct LogfilePartition {

}

impl LogfilePartition {

	pub fn create() -> Result<LogfilePartition, ::Error> {
		debug!("Creating new logfile partition");

		let part = LogfilePartition {};
		return Ok(part);
	}

	pub fn append_measurement(
			&self,
			measurement: &Measurement) -> Result<(), ::Error> {
		debug!("Storing new measurement; time={}", measurement.time);
		return Ok(());
	}

	pub fn delete(&self) -> Result<(), ::Error> {
		debug!("Deleting logfile partition");
		return Ok(());
	}

	pub fn get_storage_used_bytes(&self) -> u64 {
		return 0;
	}

}

