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
use std::mem::transmute;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Measurement {
	pub time: u64,
	pub data: Vec<u8>
}

impl Measurement {

	pub fn encode(&self) -> Vec<u8> {
		let time_encoded : [u8; 8] = unsafe {
			transmute(self.time.to_le())
		};

		let data_size_encoded : [u8; 4] = unsafe {
			transmute((self.data.len() as u32).to_le())
		};

		let mut encoded = self.data.clone();
		encoded.extend_from_slice(&data_size_encoded);
		encoded.extend_from_slice(&time_encoded);

		return encoded;
	}

	pub fn get_encoded_size(&self) -> u64 {
		return (self.data.len() + 12) as u64;
	}


}

