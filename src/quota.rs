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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageQuota {
	Unlimited,
	Limited { limit_bytes: u64 },
	Zero,
}

impl StorageQuota {
	pub fn parse_string(string: &str) -> Result<StorageQuota, ::Error> {
		let (value, modifier) = match string.rfind(char::is_numeric) {
			Some(len) => (Some(&string[0..len + 1]), &string[len + 1..]),
			None => (None, string),
		};

		let value = match value {
			Some(v) => Some(v.parse::<u64>()?),
			None => None,
		};

		match (value, modifier) {
			(None, "unlimited") | (None, "infinite") => Ok(StorageQuota::Unlimited),
			(None, "none") | (None, "zero") => Ok(StorageQuota::Zero),
			(Some(value), "KB") | (Some(value), "kb") => Ok(StorageQuota::Limited {
				limit_bytes: value * 1_000,
			}),
			(Some(value), "KiB") | (Some(value), "kib") => Ok(StorageQuota::Limited {
				limit_bytes: value * (1 << 10),
			}),
			(Some(value), "MB") | (Some(value), "mb") => Ok(StorageQuota::Limited {
				limit_bytes: value * 1_000_000,
			}),
			(Some(value), "MiB") | (Some(value), "mib") => Ok(StorageQuota::Limited {
				limit_bytes: value * (1 << 20),
			}),
			(Some(value), "GB") | (Some(value), "gb") => Ok(StorageQuota::Limited {
				limit_bytes: value * 1_000_000_000,
			}),
			(Some(value), "GiB") | (Some(value), "gib") => Ok(StorageQuota::Limited {
				limit_bytes: value * (1 << 30),
			}),
			_ => Err(err_user!("invalid quota specification: {}", string)),
		}
	}

	pub fn is_zero(&self) -> bool {
		match self {
			StorageQuota::Unlimited => false,
			&StorageQuota::Limited { limit_bytes } => limit_bytes == 0,
			StorageQuota::Zero => true,
		}
	}

	pub fn is_sufficient_bytes(&self, bytes: u64) -> bool {
		match self {
			StorageQuota::Unlimited => true,
			&StorageQuota::Limited { limit_bytes } => limit_bytes >= bytes,
			StorageQuota::Zero => false,
		}
	}
}
