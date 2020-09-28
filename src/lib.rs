/**
 * Copyright © 2018 nyantec GmbH <oss@nyantec.com>
 * Authors:
 *   Paul Asmuth <asm@nyantec.com>
 *   Karl Engelhardt <ken@nyantec.com>
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
extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate md5;

#[macro_use]
pub mod error;
pub mod logfile;
pub mod logfile_config;
pub mod logfile_directory;
pub mod logfile_id;
pub mod logfile_map;
pub mod logfile_partition;
pub mod logfile_reader;
pub mod logfile_transaction;
pub mod logfile_writer;
pub mod measure;
pub mod quota;
pub mod time;

use error::{Error, ErrorCode};
use logfile_config::LogfileConfig;
use logfile_directory::LogfileDirectory;
use logfile_id::LogfileID;
use logfile_map::LogfileMap;
use measure::Measurement;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Sensorlog {
	pub logfile_map: LogfileMap,
}

impl Sensorlog {
	pub fn new(datadir: &Path, logfile_config: LogfileConfig) -> Result<Self, ::Error> {
		if !datadir.exists() {
			return Err(err_user!("data directory does not exist: {:?}", datadir));
		}

		let logfile_directory = LogfileDirectory::open(&datadir)?;
		let logfile_map = LogfileMap::open(logfile_directory, logfile_config)?;

		let service = Self { logfile_map };

		Ok(service)
	}

	pub fn store_measurement(
		&self,
		time: Option<u64>,
		sensor_id: &str,
		data: &str,
	) -> Result<(), ::Error> {
		debug!("Storing measurement: sensor_id={}", sensor_id);

		let measurement = Measurement {
			time: time.unwrap_or(::time::get_unix_microseconds()?),
			data: data.to_string(),
		};

		let logfile_id = LogfileID::from_string(sensor_id.to_owned());
		let logfile = self.logfile_map.lookup_or_create(&logfile_id)?;
		logfile.append_measurement(&measurement)?;

		Ok(())
	}

	pub fn fetch_measurements(
		&self,
		sensor_id: &str,
		time_start: Option<u64>,
		time_limit: Option<u64>,
		limit: Option<u64>,
	) -> Result<Vec<Measurement>, ::Error> {
		let logfile_id = LogfileID::from_string(sensor_id.to_owned());

		debug!(
			"Fetching measurements: sensor_id={}; time_start={:?} time_limit={:?} limit={:?}",
			sensor_id, time_start, time_limit, limit
		);

		let measurements = match self.logfile_map.lookup(&logfile_id) {
			Some(logfile) => logfile.fetch_measurements(time_start, time_limit, limit)?,
			None => Vec::<Measurement>::new(),
		};

		Ok(measurements)
	}

	pub fn set_storage_quota_for(&mut self, sensor_id: &str, quota: ::quota::StorageQuota) -> Result<(), Error> {
		let logfile_id = LogfileID::from_string(sensor_id.to_string());
		self.logfile_map.set_storage_quota_for(&logfile_id, quota)
	}
}
