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
use std::path::{Path,PathBuf};
use std::fs;
use ::logfile_partition::LogfilePartition;
use ::logfile_transaction::LogfileTransaction;
use ::quota::StorageQuota;
use ::measure::Measurement;

const DEFAULT_PARTITION_SIZE_MAX_BYTES : u64 = 1024 * 128;
const TRANSACTION_FILE_NAME : &'static str = "tx.lock";

pub struct Logfile {
	storage: Arc<RwLock<LogfileStorage>>,
}

pub struct LogfileStorage {
	path: PathBuf,
	storage_quota: StorageQuota,
	partitions: Vec<LogfilePartition>,
	partitions_deleted: Vec<LogfilePartition>,
	partition_size_bytes: u64,
}

impl Logfile {

	pub fn create(
			path: &Path,
			storage_quota: StorageQuota) -> Result<Logfile, ::Error> {
		if storage_quota.is_zero() {
			return Err(err_quota!("insufficient quota"));
		}

		debug!("Creating new logfile");
		fs::create_dir_all(path)?;

		let logfile = Logfile {
			storage: Arc::new(RwLock::new(LogfileStorage {
				path: path.to_owned(),
				storage_quota: storage_quota,
				partitions: Vec::<LogfilePartition>::new(),
				partitions_deleted: Vec::<LogfilePartition>::new(),
				partition_size_bytes: DEFAULT_PARTITION_SIZE_MAX_BYTES
			})),
		};

		return Ok(logfile);
	}

	pub fn append_measurement(
			&self,
			measurement: &Measurement) -> Result<(), ::Error> {
		let measurement_size = measurement.get_encoded_size() as u64;

		// lock the storage
		let mut storage_locked = match self.storage.write() {
			Ok(l) => l,
			Err(_) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		// check if the measurement exceeds the total storage quota
		let quota = storage_locked.storage_quota.clone();
		if !quota.is_sufficient_bytes(measurement_size) {
			return Err(err_quota!("insufficient quota"));
		}

		// check that the measurement time is monotonically increasing
		if let Some(partition) = storage_locked.partitions.last() {
			if measurement.time < partition.get_time_head() {
				return Err(
						err_user!(
								"measurement time values must be monotonically increasing for \
								each sensor_id"));
			}
		};

		// allocate storage for the new measurement
		storage_locked.allocate(measurement_size)?;

		// insert the new measurement into the head partition
		match storage_locked.partitions.last_mut() {
			Some(p) => p.append_measurement(measurement)?,
			None => return Err(err_server!("corrupt partition map")),
		};

		// commit the transaction to disk
		return storage_locked.commit();
	}

	pub fn get_storage_quota(&self) -> StorageQuota {
		return self.storage.read().unwrap().storage_quota.clone();
	}

	pub fn set_storage_quota(&self, quota: StorageQuota) {
		self.storage.write().unwrap().storage_quota = quota;
	}

	pub fn set_partition_size_bytes(&mut self, partition_size: u64) {
		self.storage.write().unwrap().partition_size_bytes = partition_size;
	}

}

impl LogfileStorage {

	pub fn commit(&mut self) -> Result<(), ::Error> {
		// write transaction to disk
		let transaction = LogfileTransaction::from_partition_map(&self.partitions);
		let transaction_path = self.path.join(TRANSACTION_FILE_NAME);
		transaction.write_file(&transaction_path)?;

		// drop deleted partitions
		for partition in &mut self.partitions_deleted {
			partition.delete()?;
		}

		self.partitions_deleted.clear();

		return Ok(());
	}

	pub fn allocate(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		// drop partitions from the tail until the quota is met
		self.garbage_collect(new_bytes)?;

		// append a new head partition if the current head partition is full
		let new_partition = match self.partitions.last() {
			Some(partition) =>
				if partition.get_file_offset() + new_bytes > self.partition_size_bytes {
					Some(LogfilePartition::create(partition.get_time_head())?)
				} else {
					None
				},
			None => Some(LogfilePartition::create(0)?)
		};

		if let Some(partition) = new_partition {
			self.partitions.push(partition);
		}

		return Ok(());
	}

	pub fn garbage_collect(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		let mut required_bytes : u64 =
				new_bytes + self
						.partitions
						.iter()
						.fold(0, |s, x| s + x.get_file_offset());

		while !self.storage_quota.is_sufficient_bytes(required_bytes) {
			if self.partitions.len() == 0 {
				return Err(err_server!("corrupt partition map"));
			}

			let deleted_partition = self.partitions.remove(0);
			required_bytes -= deleted_partition.get_file_offset();
			self.partitions_deleted.push(deleted_partition);
		}

		return Ok(());
	}

}

