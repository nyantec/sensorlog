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
use std::path::{Path,PathBuf};
use ::measure::Measurement;

pub struct LogfilePartition {
	path: PathBuf,
	time_head: u64,
	time_tail: u64,
	offset: u64,
}

impl LogfilePartition {

	pub fn create(path: &Path, time: u64) -> Result<LogfilePartition, ::Error> {
		info!("Creating new logfile partition");

		let part = LogfilePartition {
			path: path.to_owned(),
			time_head: time,
			time_tail: time,
			offset: 0,
		};

		return Ok(part);
	}

	pub fn append_measurement(
			&mut self,
			measurement: &Measurement) -> Result<(), ::Error> {
		if measurement.time < self.time_head {
			return Err(
					err_user!(
							"measurement time values must be monotonically increasing for \
							each sensor_id"));
		}

		debug!(
				"Storing new measurement; time={}, foffset={}",
				measurement.time,
				self.offset);

		let measurement_size = measurement.get_encoded_size();
		self.time_head = measurement.time;
		self.offset += measurement_size;
		return Ok(());
	}

	pub fn delete(&self) -> Result<(), ::Error> {
		info!("Deleting logfile partition");
		return Ok(());
	}

	pub fn get_file_name(&self) -> String {
		return format!("{}.log", self.time_head);
	}

	pub fn get_file_offset(&self) -> u64 {
		return self.offset;
	}

	pub fn get_time_head(&self) -> u64 {
		return self.time_head;
	}

	pub fn get_time_tail(&self) -> u64 {
		return self.time_tail;
	}

}

