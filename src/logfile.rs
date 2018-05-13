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

const DEFAULT_PARTITION_SIZE_MAX_BYTES : u64 = 1024 * 128;

pub struct Logfile {
	state: Arc<RwLock<LogfileState>>,
}

pub struct LogfileState {
	storage_quota: StorageQuota,
	partitions: Vec<LogfilePartition>,
	partition_size_bytes: u64,
}

impl Logfile {

	pub fn create(storage_quota: StorageQuota) -> Result<Logfile, ::Error> {
		if storage_quota.is_zero() {
			return Err(err_quota!("insufficient quota"));
		}

		debug!("Creating new logfile");
		let logfile = Logfile {
			state: Arc::new(RwLock::new(LogfileState {
				storage_quota: storage_quota,
				partitions: Vec::<LogfilePartition>::new(),
				partition_size_bytes: DEFAULT_PARTITION_SIZE_MAX_BYTES
			})),
		};

		return Ok(logfile);
	}

	pub fn append_measurement(
			&self,
			measurement: &Measurement) -> Result<(), ::Error> {
		let measurement_size = measurement.get_encoded_size() as u64;

		// lock the state
		let mut state_locked = match self.state.write() {
			Ok(l) => l,
			Err(_) => {
				error!("lock is poisoned; aborting...");
				process::abort();
			}
		};

		// check the quota and that the measurement time is monotonically increasing
		let quota = state_locked.storage_quota.clone();
		if !quota.is_sufficient_bytes(measurement_size) {
			return Err(err_quota!("insufficient quota"));
		}

		if measurement.time < state_locked.get_time_head() {
			return Err(
					err_user!(
							"measurement time values must be monotonically increasing for \
							each sensor_id"));
		}

		// allocate storage for the new measurement
		state_locked.allocate_storage(measurement_size)?;

		// insert the new measurement into the head partition
		return match state_locked.partitions.last_mut() {
			Some(p) => p.append_measurement(measurement),
			None => Err(err_server!("corrupt partition map")),
		};
	}

	pub fn get_storage_quota(&self) -> StorageQuota {
		return self.state.read().unwrap().storage_quota.clone();
	}

	pub fn set_storage_quota(&self, quota: StorageQuota) {
		self.state.write().unwrap().storage_quota = quota;
	}

	pub fn set_partition_size_bytes(&mut self, partition_size: u64) {
		self.state.write().unwrap().partition_size_bytes = partition_size;
	}

}

impl LogfileState {

	pub fn get_time_head(&self) -> u64 {
		return match self.partitions.last() {
			Some(p) => p.get_time_head(),
			None => 0,
		};
	}

	pub fn allocate_storage(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		// drop partitions from the tail until the quota is met
		self.rotate_storage(new_bytes)?;

		// append a new head partition if the current head partition is full
		let new_partition = match self.partitions.last() {
			Some(partition) =>
				if partition.get_storage_used_bytes() + new_bytes > self.partition_size_bytes {
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

	pub fn rotate_storage(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		let mut required_bytes : u64 =
				new_bytes + self
						.partitions
						.iter()
						.fold(0, |s, x| s + x.get_storage_used_bytes());

		while !self.storage_quota.is_sufficient_bytes(required_bytes) {
			if self.partitions.len() == 0 {
				return Err(err_server!("corrupt partition map"));
			}

			self.partitions[0].delete()?;
			let deleted_partition = self.partitions.remove(0);
			required_bytes -= deleted_partition.get_storage_used_bytes();
		}

		return Ok(());
	}

}

