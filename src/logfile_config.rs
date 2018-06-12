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
use ::logfile_id::LogfileID;
use ::quota::StorageQuota;

const DEFAULT_PARTITION_SIZE_MAX_BYTES : u64 = 1024 * 128;

pub struct LogfileConfig {
	quota_default: StorageQuota,
	quota: HashMap<LogfileID, StorageQuota>,
	partition_size_bytes_default: u64,
}

impl LogfileConfig {

	pub fn new() -> LogfileConfig {
		return LogfileConfig {
			quota_default: StorageQuota::Zero,
			quota: HashMap::<LogfileID, StorageQuota>::new(),
			partition_size_bytes_default: DEFAULT_PARTITION_SIZE_MAX_BYTES
		};
	}

	pub fn get_storage_quota_for(&self, logfile_id: &LogfileID) -> StorageQuota {
		return self
				.quota
				.get(logfile_id)
				.unwrap_or(&self.quota_default)
				.clone();
	}

	pub fn set_storage_quota_for(
			&mut self,
			logfile_id: &LogfileID,
			quota: StorageQuota) {
		self.quota.insert(logfile_id.clone(), quota);
	}

	pub fn set_default_storage_quota(&mut self, quota: StorageQuota) {
		self.quota_default = quota;
	}

	pub fn get_partition_size_for(&self, _logfile_id: &LogfileID) -> u64 {
		return self.partition_size_bytes_default;
	}

	pub fn set_default_partition_size_bytes(&mut self, limit: u64) {
		self.partition_size_bytes_default = limit;
	}

}

