/**
 * Copyright © 2018 nyantec GmbH <oss@nyantec.com>
 * Authors:
 *   Paul Asmuth <asm@nyantec.com>
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
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate md5;

#[macro_use] mod error;
mod api;
mod api_json;
mod http;
mod logfile;
mod logfile_id;
mod logfile_config;
mod logfile_directory;
mod logfile_map;
mod logfile_partition;
mod logfile_transaction;
mod logfile_reader;
mod logfile_writer;
mod quota;
mod measure;
mod service;
mod time;

use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use ::error::{Error,ErrorCode};

const VERSION : &'static str = env!("CARGO_PKG_VERSION");
const LOGLEVEL_DEFAULT : &'static str = "info";

const USAGE : &'static str = "\
Usage: $ sensorlogd [OPTIONS]

Options:

   --listen_http=<addr>
      Listen for HTTP connection on this address

   --datadir=<dir>
      Set the data directory

   --quota_default=<quota>
      Set the default storage quota for all sensors

   --quota=<sensor_id>:<quota>
       Set the storage quota for a given sensor id

   --clock_watchdog=<mode>
      Enable the clock watchdog. Modes are 'off', 'panic' and 'wipe'

   --clock_watchdog_trigger_forward=<threshold>
      Trigger the clock watchdog if the system time jumps forward by more than threshold

   --clock_watchdog_trigger_backward=<threshold>
      Trigger the clock watchdog if the system time jumps backward by more than threshold

   --partition_size=<bytes>
      Set the partition size (default: 128KB)

   --daemonize
      Daemonize the server

   --pidfile=<file>
      Write a PID file

   --loglevel <level>
      Minimum log level (default: INFO)

   --[no]log_to_syslog
      Do[n't] log to syslog

   --[no]log_to_stderr
      Do[n't] log to stderr

   -?, --help
      Display this help text and exit

   -V, --version
      Display the version of this binary and exit

Examples:
   $ sensorlogd --datadir /var/sensordata --listen_http localhost:8080 --quota_default infinite
";

fn main() {
	if let Err(err) = run() {
		writeln!(&mut std::io::stderr(), "ERROR: {}", err).unwrap();
		std::process::exit(1);
	}
}

fn run() -> Result<(), ::Error> {
	// parse command line flags
	let args : Vec<String> = env::args().collect();

	let mut flag_cfg = getopts::Options::new();
	flag_cfg.optopt("", "listen_http", "", "PORT");
	flag_cfg.optopt("", "datadir", "", "PATH");
	flag_cfg.optopt("", "quota_default", "", "QUOTA");
	flag_cfg.optmulti("", "quota", "", "QUOTA");
	flag_cfg.optopt("", "partition_size", "", "BYTES");
	flag_cfg.optopt("", "loglevel", "", "LEVEL");
	flag_cfg.optflag("h", "help", "");
	flag_cfg.optflag("v", "version", "");

	let flags = match flag_cfg.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => {
			writeln!(
					&mut std::io::stderr(),
					"invalid command line options: {}",
					f.to_string()).unwrap();

			std::process::exit(1);
		}
	};

	if flags.opt_present("h") {
		std::io::stdout().write(USAGE.as_bytes()).unwrap();
		return Ok(());
	}

	// start logger
	std::env::set_var(
			"RUST_LOG",
			format!(
					"sensorlog={},{}",
					flags.opt_str("loglevel").unwrap_or(LOGLEVEL_DEFAULT.into()),
					std::env::var("RUST_LOG").unwrap_or("".into())));

	env_logger::init();

	info!("sensorlog v{}", VERSION);

	// configure storage quotas
	let mut logfile_config = logfile_config::LogfileConfig::new();

	logfile_config.set_default_storage_quota(
			quota::StorageQuota::parse_string(
					&match &flags.opt_str("quota_default") {
						&Some(ref v) => v,
						&None => return Err(err_user!("missing option: --quota_default"))
					})?);

	for quota_opt in &flags.opt_strs("quota") {
		let (sensor_id, sensor_quota) = match quota_opt.find(':') {
			Some(len) => (&quota_opt[0..len], &quota_opt[len+1..]),
			None => return Err(err_user!("invalid --quota flag: {}", quota_opt)),
		};

		logfile_config.set_storage_quota_for(
				&logfile_id::LogfileID::from_string(sensor_id.to_string()),
				quota::StorageQuota::parse_string(sensor_quota)?);
	}

	if let Some(partition_size) = flags.opt_str("partition_size") {
		logfile_config.set_default_partition_size_bytes(
				match partition_size.parse::<u64>() {
					Ok(v) => v,
					Err(e) => return Err(err_user!("invalid partition size: {}", e))
				});
	}

	// start logging service
	let datadir = match flags.opt_str("datadir") {
		Some(v) => PathBuf::from(v),
		None => return Err(err_user!("missing option: --datadir"))
	};

	let service = Arc::new(service::Service::start(&datadir, logfile_config)?);

	// start http server
	http::start_server(
			service,
			http::ServerOptions {
				listen_addr: match flags.opt_str("listen_http") {
					Some(addr) => addr,
					None => "[::]:8080".to_owned()
				},
			})?;

	return Ok(());
}

