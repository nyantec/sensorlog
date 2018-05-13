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
use std::path::Path;
use std::fs;
use std::io::Write;
use serde_json as json;
use ::logfile_partition::LogfilePartition;

#[derive(Serialize, Deserialize)]
pub struct LogfileTransaction {
	partitions: Vec<LogfileTransactionPartition>,
}

#[derive(Serialize, Deserialize)]
pub struct LogfileTransactionPartition {
	file_name: String,
	file_offset: u64,
	time_head: u64,
	time_tail: u64
}

impl LogfileTransaction {

	pub fn from_partition_map(partitions: &Vec<LogfilePartition>) -> LogfileTransaction {
		let partitions = partitions.iter().map(|partition| {
			return LogfileTransactionPartition {
				file_name: partition.get_file_name(),
				file_offset: partition.get_file_offset(),
				time_head: partition.get_time_head(),
				time_tail: partition.get_time_tail(),
			}
		});

		return LogfileTransaction {
			partitions: partitions.collect()
		};
	}

	pub fn write_file(&self, path: &Path) -> Result<(), ::Error> {
		let encoded = match json::to_vec(&self) {
			Ok(v) => v,
			Err(e) => return Err(err_server!("error while encoding JSON: {}", e))
		};

		let path_swap = format!("{}.swap", match path.to_str() {
			Some(v) => v,
			None => return Err(err_server!("invalid transaction path"))
		});

		// write to swap file
		{
			let mut file = fs::File::create(&path_swap)?;
			file.write_all(&encoded)?;
			file.sync_data()?;
		}

		// replace transaction file with swap file
		fs::rename(&path_swap, &path)?;

		return Ok(());
	}

}


