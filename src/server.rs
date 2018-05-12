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
extern crate iron;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

#[macro_use] mod error;
mod http;
mod api;
mod api_json;

use std::env;
use std::io;
use std::io::Write;
use ::error::{Error,ErrorCode};

const VERSION : &'static str = env!("CARGO_PKG_VERSION");
const LOGLEVEL_DEFAULT : &'static str = "info";

const USAGE : &'static str = "\
Usage: $ esensord [OPTIONS]
   --listen_http <addr>          Listen for HTTP connection on this address
   --datadir <dir>               Set the data directory
   --quota_default <quota>       Set the default storage quota for all sensors
   --quota <sensor_id>:<quota>   Set the storage quota for a given sensor id
   --daemonize                   Daemonize the server
   --pidfile <file>              Write a PID file
   --loglevel <level>            Minimum log level (default: INFO)
   --[no]log_to_syslog           Do[n't] log to syslog
   --[no]log_to_stderr           Do[n't] log to stderr
   -?, --help                    Display this help text and exit
   -V, --version                 Display the version of this binary and exit

Examples:
   $ esensord --datadir /var/sensordata --listen_http localhost:8080 --quota_default infinite
";

fn main() {
	// parse command line flags
	let args : Vec<String> = env::args().collect();

	let mut flag_cfg = getopts::Options::new();
	flag_cfg.optopt("", "listen_http", "", "PORT");
	flag_cfg.optopt("", "datadir", "", "PATH");
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
		return;
	}

	// start logger
	let loglevel = flags.opt_str("loglevel").unwrap_or(LOGLEVEL_DEFAULT.into());
	std::env::set_var("RUST_LOG", loglevel);
	env_logger::init();

	// start server
	info!("esensord v{}", VERSION);

	let datadir = match flags.opt_str("datadir") {
		Some(v) => v,
		None => {
			writeln!(&mut std::io::stderr(), "missing option: --datadir").unwrap();
			std::process::exit(1);
		}
	};

	// start http server
	http::http_server_start(http::ServerOptions {
		listen_addr: match flags.opt_str("listen_http") {
			Some(addr) => addr,
			None => "[::]:8080".to_owned()
		},
	});

}

