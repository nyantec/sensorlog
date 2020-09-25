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
use logfile_partition::LogfilePartition;
use measure::Measurement;
use std::fs;

#[derive(Debug, Clone)]
pub struct LogfileReader<'a> {
	partitions: &'a [LogfilePartition],
}

impl<'a> LogfileReader<'a> {
	pub fn new(partitions: &'a [LogfilePartition]) -> LogfileReader<'a> {
		LogfileReader { partitions }
	}

	pub fn fetch_measurements(
		&self,
		time_start: Option<u64>,
		time_limit: Option<u64>,
		limit: Option<u64>,
	) -> Result<Vec<Measurement>, ::Error> {
		let mut measurements = Vec::<Measurement>::new();

		'scan: for partition in self.partitions.iter().rev() {
			// skip partitions that are not part of the time range
			if let Some(time_start) = time_start {
				if partition.get_time_tail() > time_start {
					continue;
				}
			}

			// break if there is no further partition that can match the time range
			if let Some(time_limit) = time_limit {
				if partition.get_time_head() <= time_limit {
					break 'scan;
				}
			}

			let mut file = fs::File::open(partition.get_file_path())?;
			let mut file_offset = partition.get_file_offset();

			while file_offset > 0 {
				let measurement = Measurement::decode(&mut file, file_offset)?;

				if measurement.get_encoded_size() <= file_offset {
					file_offset -= measurement.get_encoded_size();
				} else {
					return Err(err_server!("corrupt file"));
				}

				if let Some(time_start) = time_start {
					// skip measurements that are not part of the time range
					if measurement.time > time_start {
						continue;
					}
				}

				// break once the end of the time window is reached
				if let Some(time_limit) = time_limit {
					if measurement.time <= time_limit {
						break 'scan;
					}
				}

				measurements.push(measurement);

				// break once limit is reached
				if let Some(limit) = limit {
					if measurements.len() as u64 == limit {
						break 'scan;
					}
				}
			}
		}

		Ok(measurements)
	}
}
