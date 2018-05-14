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
use std::fs;
use std::io::{Read,Seek,SeekFrom};
use ::logfile_partition::LogfilePartition;
use ::measure::Measurement;

pub struct LogfileReader<'a> {
	path: PathBuf,
	partitions: &'a Vec<LogfilePartition>,
}

impl<'a> LogfileReader<'a> {

	pub fn new(path: &Path, partitions: &'a Vec<LogfilePartition>) -> LogfileReader<'a> {
		return LogfileReader {
			path: path.to_owned(),
			partitions: partitions
		};
	}

	pub fn fetch_last_measurement(&self) -> Result<Option<Measurement>, ::Error> {
		let partition = match self.partitions.last() {
			Some(p) => p,
			None => return Ok(None)
		};

		let mut file = fs::File::open(partition.get_file_path())?;
		let measurement = Measurement::decode(
				&mut file,
				partition.get_file_offset())?;

		return Ok(Some(measurement));
	}

}
