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
use std::sync::{Arc,Mutex,RwLock};
use std::process;
use ::logfile_partition::LogfilePartition;
use ::quota::StorageQuota;
use ::measure::Measurement;

pub struct Logfile {
	storage_quota: Mutex<StorageQuota>,
	partition_map: Arc<RwLock<LogfilePartitionMap>>,
	partition_size_max_bytes: u64,
}

pub struct LogfilePartitionMap {
	partitions: Vec<LogfilePartition>,
}

impl Logfile {

	pub fn create(
			storage_quota: StorageQuota,
			partition_size_max_bytes: u64) -> Result<Logfile, ::Error> {
		if storage_quota.is_zero() {
			return Err(err_quota!("insufficient quota"));
		}

		debug!("Creating new logfile");
		let logfile = Logfile {
			storage_quota: Mutex::new(storage_quota),
			partition_map: Arc::new(RwLock::new(LogfilePartitionMap {
				partitions: Vec::<LogfilePartition>::new(),
			})),
			partition_size_max_bytes: partition_size_max_bytes
		};

		return Ok(logfile);
	}

	pub fn append_measurement(
			&self,
			measurement: &Measurement) -> Result<(), ::Error> {
		let quota = self.get_storage_quota();
		if !quota.is_sufficient_bytes(measurement.get_encoded_size() as u64) {
			return Err(err_quota!("insufficient quota"));
		}

		// lock the partition map
		let mut pmap_locked = match self.partition_map.write() {
			Ok(l) => l,
			Err(_) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		// drop partitions from the tail until the quota is met
		let mut required_bytes : u64 =
				measurement.get_encoded_size() +
				pmap_locked
						.partitions
						.iter()
						.fold(0, |s, x| s + x.get_storage_used_bytes());

		while !quota.is_sufficient_bytes(required_bytes) {
			if pmap_locked.partitions.len() == 0 {
				return Err(err_server!("corrupt partition map"));
			}

			pmap_locked.partitions[0].delete()?;
			let deleted_partition = pmap_locked.partitions.remove(0);
			required_bytes -= deleted_partition.get_storage_used_bytes();
		}

		// append a new head partition if the current head partition is full
		let head_partition_full = match pmap_locked.partitions.last() {
			Some(p) => false, // FIXME
			None => true,
		};

		if head_partition_full {
			let new_partition = LogfilePartition::create()?;
			pmap_locked.partitions.push(new_partition);
		}

		// insert the new measurement into the new head partition
		let head_partition = match pmap_locked.partitions.last_mut() {
			Some(p) => p,
			None => return Err(err_server!("corrupt partition map")),
		};

		return head_partition.append_measurement(measurement);
	}

	pub fn get_storage_quota(&self) -> StorageQuota {
		return self.storage_quota.lock().unwrap().clone();
	}

	pub fn set_storage_quota(&self, quota: StorageQuota) {
		*self.storage_quota.lock().unwrap() = quota;
	}

}

