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
use ::logfile_map::LogfileMap;
use ::logfile_directory::LogfileDirectory;
use ::logfile_config::LogfileConfig;

pub struct Service {
	pub logfile_map: LogfileMap,
}

impl Service {

	pub fn start(
			datadir: &Path,
			logfile_config: LogfileConfig) -> Result<Service, ::Error> {
		if !datadir.exists() {
			return Err(err_user!("data directory does not exist: {:?}", datadir));
		}

		let logfile_directory = LogfileDirectory::open(&datadir)?;
		let logfile_map = LogfileMap::open(logfile_directory, logfile_config)?;

		let service = Service {
			logfile_map: logfile_map
		};

		return Ok(service);
	}

}

