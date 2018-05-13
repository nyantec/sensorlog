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
use std::sync::{Arc,Mutex};
use ::logfile_partition::LogfilePartition;
use ::quota::StorageQuota;

pub struct Logfile {
	storage_quota: Mutex<StorageQuota>
}

impl Logfile {

	pub fn create(storage_quota: StorageQuota) -> Result<Logfile, ::Error> {
		if storage_quota.is_zero() {
			return Err(err_quota!("insufficient quota"));
		}

		debug!("Creating new logfile");
		let logfile = Logfile {
			storage_quota: Mutex::new(storage_quota)
		};

		return Ok(logfile);
	}

	pub fn append_measurement(
			&self,
			time: &Option<u64>,
			data: &[u8]) -> Result<(), ::Error> {
		return Err(err_server!("nyi"));
	}

	pub fn set_storage_quota(&self, quota: StorageQuota) {
		*self.storage_quota.lock().unwrap() = quota;
	}

}
