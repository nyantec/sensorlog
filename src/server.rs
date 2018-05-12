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
extern crate env_logger;

#[macro_use]
extern crate log;

use std::env;
use std::io;
use std::io::Write;

const VERSION : &'static str = env!("CARGO_PKG_VERSION");
const LOGLEVEL_DEFAULT : &'static str = "info";

fn print_usage() {

}

fn main() {
	// parse command line flags
	let args : Vec<String> = env::args().collect();

	let mut flag_cfg = getopts::Options::new();
	flag_cfg.optopt("", "listen_http", "Listen for http connections", "PORT");
	flag_cfg.optopt("", "loglevel", "Loglevel", "LEVEL");
	flag_cfg.optflag("h", "help", "Display this help text and exit");
	flag_cfg.optflag("v", "version", "Display the version of this binary and exit");

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
		print_usage();
		return;
	}

	// start logger
	let loglevel = flags.opt_str("loglevel").unwrap_or(LOGLEVEL_DEFAULT.into());
	std::env::set_var("RUST_LOG", loglevel);
	env_logger::init();

	// start server
	info!("esensord v{}", VERSION);
}

