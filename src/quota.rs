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
#[derive(Debug, Clone)]
pub enum StorageQuota {
	Unlimited,
	Limited{limit_bytes: u64},
	Zero
}

impl StorageQuota {

	pub fn parse_string(string: &str) -> Result<StorageQuota, ::Error> {
		return match string {
			"unlimited" | "infinite" => Ok(StorageQuota::Unlimited),
			"none" | "zero" => Ok(StorageQuota::Zero),
			s => match s.parse::<u64>() {
				Ok(v) => Ok(StorageQuota::Limited{limit_bytes: v}),
				Err(e) => Err(err_user!("invalid storage quota specification"))
			}
		};
	}

	pub fn is_zero(&self) -> bool {
		return match self {
			&StorageQuota::Unlimited => false,
			&StorageQuota::Limited{limit_bytes} => limit_bytes == 0,
			&StorageQuota::Zero => true,
		};
	}

	pub fn is_sufficient_bytes(&self, bytes: u64) -> bool {
		return match self {
			&StorageQuota::Unlimited => true,
			&StorageQuota::Limited{limit_bytes} => limit_bytes >= bytes,
			&StorageQuota::Zero => false,
		};
	}

}
