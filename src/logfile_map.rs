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
use std::path::{Path,PathBuf};
use std::process;
use ::logfile::Logfile;
use ::quota::StorageQuota;

pub struct LogfileMap {
	path: PathBuf,
	logfiles: Arc<RwLock<HashMap<String, Arc<Logfile>>>>,
	quota_default: StorageQuota,
	partition_size_bytes_default: Option<u64>,
}

impl LogfileMap {

	fn new(path: &Path) -> LogfileMap {
		return LogfileMap {
			path: path.to_owned(),
			logfiles: Arc::new(RwLock::new(HashMap::<String, Arc<Logfile>>::new())),
			quota_default: StorageQuota::Zero,
			partition_size_bytes_default: None
		};
	}

	pub fn open(path: &Path) -> Result<LogfileMap, ::Error> {
		info!("Opening logfile database at {:?}", path);

		let logfile_map = LogfileMap::new(path);
		return Ok(logfile_map);
	}

	pub fn lookup(self: &LogfileMap, logfile_id: &str) -> Option<Arc<Logfile>> {
		let logfiles_locked = match self.logfiles.read() {
			Ok(l) => l,
			Err(_) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		return logfiles_locked.get(logfile_id).map(|l| l.clone());
	}

	pub fn lookup_or_create(
			self: &LogfileMap,
			logfile_id: &str) -> Result<Arc<Logfile>, ::Error> {
		// rust RWLocks don't support upgrades. so we implement an optimistic
		// fast path using a read lock
		if let Some(logfile) = self.lookup(logfile_id) {
			return Ok(logfile);
		}

		// grab write lock
		let mut logfiles_locked = match self.logfiles.write() {
			Ok(l) => l,
			Err(_) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		// check if the logfile exists again (pessimistic case)
		if let Some(logfile) = logfiles_locked.get(logfile_id) {
			return Ok(logfile.clone());
		}

		// if the logfile doesn't exist yet, create a new one
		let logfile = self.create_logfile(logfile_id)?;
		logfiles_locked.insert(logfile_id.to_string(), logfile.clone());
		return Ok(logfile);
	}

	pub fn create_logfile(&self, logfile_id: &str) -> Result<Arc<Logfile>, ::Error> {
		let mut logfile = Logfile::create(self.quota_default.clone())?;

		if let Some(partition_size) = self.partition_size_bytes_default {
			logfile.set_partition_size_bytes(partition_size);
		}

		return Ok(Arc::new(logfile));
	}

	pub fn set_storage_quota(
			&mut self,
			logfile_id: &str,
			quota: StorageQuota) -> Result<(), ::Error> {
		let logfile = self.lookup_or_create(logfile_id)?;
		logfile.set_storage_quota(quota);
		return Ok(());
	}

	pub fn set_default_storage_quota(&mut self, quota: StorageQuota) {
		self.quota_default = quota;
	}

	pub fn set_default_partition_size_bytes(&mut self, limit: u64) {
		self.partition_size_bytes_default = Some(limit);
	}

}

