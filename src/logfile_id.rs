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
use md5;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LogfileID {
	id: String,
}

pub struct LogfilePath {
	file_name: String,
}

impl LogfileID {
	pub fn from_string(string: String) -> LogfileID {
		LogfileID { id: string }
	}

	pub fn get_string(&self) -> String {
		self.id.to_owned()
	}

	pub fn get_path(&self) -> LogfilePath {
		let id_digest = md5::compute(self.id.as_bytes());

		LogfilePath {
			file_name: format!("{:x}", id_digest),
		}
	}
}

impl LogfilePath {
	pub fn from_file_name(file_name: String) -> LogfilePath {
		LogfilePath { file_name }
	}

	pub fn get_file_name(&self) -> String {
		self.file_name.to_owned()
	}
}
