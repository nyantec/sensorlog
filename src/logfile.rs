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
use logfile_config::LogfileConfig;
use logfile_id::LogfileID;
use logfile_partition::LogfilePartition;
use logfile_reader::LogfileReader;
use logfile_transaction::LogfileTransaction;
use measure::Measurement;
use quota::StorageQuota;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

const TRANSACTION_FILE_NAME: &str = "tx.lock";

#[derive(Debug, Clone)]
pub struct Logfile {
	storage: Arc<RwLock<LogfileStorage>>,
}

#[derive(Debug, Clone)]
pub struct LogfileStorage {
	id: LogfileID,
	path: PathBuf,
	storage_quota: StorageQuota,
	partitions: Vec<LogfilePartition>,
	partitions_deleted: Vec<LogfilePartition>,
	partition_size_bytes: u64,
}

impl Logfile {
	pub fn create(id: LogfileID, path: &Path, config: &LogfileConfig) -> Result<Logfile, ::Error> {
		let storage_quota = config.get_storage_quota_for(&id);
		if storage_quota.is_zero() {
			return Err(err_quota!("insufficient quota"));
		}

		debug!("Creating new logfile; id={:?}", id);
		fs::create_dir_all(path)?;

		let logfile = Logfile {
			storage: Arc::new(RwLock::new(LogfileStorage {
				id: id.clone(),
				path: path.to_owned(),
				storage_quota,
				partitions: Vec::<LogfilePartition>::new(),
				partitions_deleted: Vec::<LogfilePartition>::new(),
				partition_size_bytes: config.get_partition_size_for(&id),
			})),
		};

		Ok(logfile)
	}

	pub fn open(path: &Path, config: &LogfileConfig) -> Result<Option<Logfile>, ::Error> {
		let transaction_path = path.join(TRANSACTION_FILE_NAME).to_owned();
		if !transaction_path.exists() {
			return Ok(None);
		}

		let transaction = LogfileTransaction::read_file(&transaction_path)?;

		debug!("Loading logfile; id={:?}", transaction.id);

		let logfile_id = LogfileID::from_string(transaction.id);
		let mut logfile_partitions = Vec::<LogfilePartition>::new();

		for partition in transaction.partitions {
			logfile_partitions.push(LogfilePartition::open(
				path,
				partition.time_head,
				partition.time_tail,
				partition.offset,
			));
		}

		let logfile = Logfile {
			storage: Arc::new(RwLock::new(LogfileStorage {
				id: logfile_id.clone(),
				path: path.to_owned(),
				storage_quota: config.get_storage_quota_for(&logfile_id),
				partitions: logfile_partitions,
				partitions_deleted: Vec::<LogfilePartition>::new(),
				partition_size_bytes: config.get_partition_size_for(&logfile_id),
			})),
		};

		Ok(Some(logfile))
	}

	pub fn get_id(&self) -> LogfileID {
		let storage_locked = match self.storage.read() {
			Ok(l) => l,
			Err(_) => fatal!("lock is poisoned"),
		};

		storage_locked.id.clone()
	}

	pub fn append_measurement(&self, measurement: &Measurement) -> Result<(), ::Error> {
		let measurement_size = measurement.get_encoded_size() as u64;

		// lock the storage
		let mut storage_locked = match self.storage.write() {
			Ok(l) => l,
			Err(_) => fatal!("lock is poisoned"),
		};

		// check if the measurement exceeds the total storage quota
		let quota = storage_locked.storage_quota.clone();
		if !quota.is_sufficient_bytes(measurement_size) {
			return Err(err_quota!("insufficient quota"));
		}

		// check that the measurement time is monotonically increasing
		let is_monotonic = match storage_locked.partitions.last() {
			Some(p) => measurement.time >= p.get_time_head(),
			None => true,
		};

		if !is_monotonic {
			warn!(
				"Clock for sensor {:?} jumped backwards, flushing data...",
				storage_locked.id
			);

			storage_locked.clear()?;
			storage_locked.commit()?;
		}

		// allocate storage for the new measurement
		storage_locked.allocate(measurement_size)?;

		// insert the new measurement into the head partition
		match storage_locked.partitions.last_mut() {
			Some(p) => p.append_measurement(measurement)?,
			None => return Err(err_server!("corrupt partition map")),
		};

		// commit the transaction to disk
		storage_locked.commit()
	}

	pub fn fetch_measurements(
		&self,
		time_start: Option<u64>,
		time_limit: Option<u64>,
		limit: Option<u64>,
	) -> Result<Vec<Measurement>, ::Error> {
		let storage_locked = match self.storage.read() {
			Ok(l) => l,
			Err(_) => fatal!("lock is poisoned"),
		};

		let reader = LogfileReader::new(&storage_locked.partitions);
		reader.fetch_measurements(time_start, time_limit, limit)
	}
}

impl LogfileStorage {
	pub fn commit(&mut self) -> Result<(), ::Error> {
		// write transaction to disk
		let transaction = LogfileTransaction::new(&self.id, &self.partitions);

		let transaction_path = self.path.join(TRANSACTION_FILE_NAME);
		transaction.write_file(&transaction_path)?;

		// drop deleted partitions
		for partition in &mut self.partitions_deleted {
			partition.delete()?;
		}

		self.partitions_deleted.clear();

		Ok(())
	}

	pub fn allocate(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		// drop partitions from the tail until the quota is met
		self.garbage_collect(new_bytes)?;

		// append a new head partition if the current head partition is full
		let new_partition = match self.partitions.last() {
			Some(partition) => {
				if partition.get_file_offset() + new_bytes > self.partition_size_bytes {
					Some(LogfilePartition::create(
						&self.path,
						partition.get_time_head(),
					)?)
				} else {
					None
				}
			}
			None => Some(LogfilePartition::create(&self.path, 0)?),
		};

		if let Some(partition) = new_partition {
			self.partitions.push(partition);
		}

		Ok(())
	}

	pub fn clear(&mut self) -> Result<(), ::Error> {
		self.partitions_deleted.append(&mut self.partitions);
		self.partitions.clear();
		Ok(())
	}

	pub fn garbage_collect(&mut self, new_bytes: u64) -> Result<(), ::Error> {
		let mut required_bytes: u64 = new_bytes
			+ self
				.partitions
				.iter()
				.fold(0, |s, x| s + x.get_file_offset());

		while !self.storage_quota.is_sufficient_bytes(required_bytes) {
			if self.partitions.is_empty() {
				return Err(err_server!("corrupt partition map"));
			}

			let deleted_partition = self.partitions.remove(0);
			required_bytes -= deleted_partition.get_file_offset();
			self.partitions_deleted.push(deleted_partition);
		}

		Ok(())
	}
}
