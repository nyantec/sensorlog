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
use std::collections::HashMap;
use std::sync::{Arc,RwLock};
use std::path::Path;
use std::process;

pub struct Logfile {}

impl Logfile {

	fn new() -> Logfile {
		return Logfile{};
	}

}

pub struct LogfileMap {
	logfiles: Arc<RwLock<HashMap<String, Arc<Logfile>>>>,
}

impl LogfileMap {

	fn new() -> LogfileMap {
		return LogfileMap {
			logfiles: Arc::new(RwLock::new(HashMap::<String, Arc<Logfile>>::new())),
		};
	}

	pub fn open(path: &Path) -> Result<LogfileMap, ::Error> {
		let mut logfile_map = LogfileMap::new();
		return Ok(logfile_map);
	}

	pub fn lookup(self: &LogfileMap, logfile_id: &str) -> Option<Arc<Logfile>> {
		let logfiles_locked = match self.logfiles.read() {
			Ok(l) => l,
			Err(e) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		return logfiles_locked.get(logfile_id).map(|l| l.clone());
	}

	pub fn lookup_or_create(self: &LogfileMap, logfile_id: &str) -> Arc<Logfile> {
		// rust RWLocks don't support upgrades. so we implement an optimistic
		// fast path using a read lock
		if let Some(logfile) = self.lookup(logfile_id) {
			return logfile;
		}

		// grab write lock
		let mut logfiles_locked = match self.logfiles.write() {
			Ok(l) => l,
			Err(e) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		// check if the logfile exists again (pessimistic case)
		if let Some(logfile) = logfiles_locked.get(logfile_id) {
			return logfile.clone();
		}

		// if the logfile doesn't exist yet, create a new one
		let logfile = Arc::new(Logfile::new());
		logfiles_locked.insert(logfile_id.to_string(), logfile.clone());
		return logfile;
	}

}

