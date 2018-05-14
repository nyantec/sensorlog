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
use std::io::{Write,Seek,SeekFrom};
use ::measure::Measurement;

pub fn append(
		path: &Path,
		offset: u64,
		measurement: &Measurement) -> Result<u64, ::Error> {
	let mut file_opts = fs::OpenOptions::new();
	file_opts.write(true);
	file_opts.create(true);

	let mut file = file_opts.open(&path)?;
	measurement.encode(&mut file, offset)?;
	file.sync_data()?;

	return Ok(measurement.get_encoded_size());
}

