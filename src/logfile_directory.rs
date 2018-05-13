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
use std::sync::{Arc,RwLock};
use ::logfile::Logfile;
use ::logfile_id::{LogfileID, LogfileFSID};
use ::logfile_config::LogfileConfig;

pub struct LogfileDirectory {
	pub path: PathBuf,
}

impl LogfileDirectory {

	pub fn new(path: &Path) -> LogfileDirectory {
		return LogfileDirectory {
			path: path.to_owned(),
		};
	}

	pub fn create_logfile(
			&self,
			logfile_id: &LogfileID,
			logfile_config: &LogfileConfig) -> Result<Arc<Logfile>, ::Error> {
		let logfile_path = self.path
				.join("db")
				.join(logfile_id.get_file_name());

		let mut logfile = Logfile::create(
				logfile_id.clone(),
				&logfile_path,
				logfile_config)?;

		return Ok(Arc::new(logfile));
	}

	pub fn load_logfile(
			&self,
			logfile_id: &LogfileFSID,
			logfile_config: &LogfileConfig) -> Result<Option<Arc<Logfile>>, ::Error> {
		let logfile_path = self.path
				.join("db")
				.join(&logfile_id.fsid);

		let mut logfile = Logfile::open(
				&logfile_path,
				logfile_config)?;

		return Ok(logfile.map(|f| Arc::new(f)));
	}

	pub fn list_logfiles(&self) -> Result<Vec<LogfileFSID>, ::Error> {
		let mut logfiles = Vec::<LogfileFSID>::new();

		for dirent in fs::read_dir(self.path.join("db"))? {
			let dirent = dirent?;
			logfiles.push(LogfileFSID {
				fsid: dirent.file_name().into_string()?
			});
		}

		return Ok(logfiles);
	}

}


