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
use serde;
use serde::ser::SerializeStruct;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;

const FOOTER_SIZE: u64 = 12;

#[derive(Debug, Clone)]
pub struct Measurement {
	pub time: u64,
	pub data: Vec<u8>,
}

impl Measurement {
	pub fn decode<S: Read + Seek>(storage: &mut S, offset: u64) -> Result<Measurement, ::Error> {
		if offset < FOOTER_SIZE {
			return Err(err_server!("invalid offset"));
		}

		let mut footer = vec![0u8; FOOTER_SIZE as usize];
		storage.seek(SeekFrom::Start(offset - FOOTER_SIZE as u64))?;
		storage.read_exact(&mut footer)?;

		let mut time_encoded: [u8; 8] = Default::default();
		time_encoded.copy_from_slice(&footer[4..12]);

		let time = u64::from_le(unsafe { mem::transmute_copy(&time_encoded) });

		let mut data_size_encoded: [u8; 4] = Default::default();
		data_size_encoded.copy_from_slice(&footer[0..4]);

		let data_size = u32::from_le(unsafe { mem::transmute_copy(&data_size_encoded) });

		if offset < FOOTER_SIZE + data_size as u64 {
			return Err(err_server!("invalid offset"));
		}

		let data_offset = offset - data_size as u64 - FOOTER_SIZE as u64;

		let mut data = vec![0; data_size as usize];

		// N.B. there doesnt appear to be a binding to pread in the rust standard lib
		storage.seek(SeekFrom::Start(data_offset))?;
		storage.read_exact(&mut data)?;

		let measurement = Measurement { time, data };

		Ok(measurement)
	}

	pub fn encode<S: Write + Seek>(&self, storage: &mut S, offset: u64) -> Result<(), ::Error> {
		let time_encoded: [u8; 8] = unsafe { mem::transmute(self.time.to_le()) };

		let data_size_encoded: [u8; 4] =
			unsafe { mem::transmute((self.data.len() as u32).to_le()) };

		let mut encoded = self.data.clone();
		encoded.extend_from_slice(&data_size_encoded);
		encoded.extend_from_slice(&time_encoded);

		assert!(encoded.len() as u64 == self.get_encoded_size());

		// N.B. there doesnt appear to be a binding to pwrite in the rust standard lib
		storage.seek(SeekFrom::Start(offset))?;
		storage.write_all(&encoded)?;
		Ok(())
	}

	pub fn get_encoded_size(&self) -> u64 {
		(self.data.len() as u64 + FOOTER_SIZE) as u64
	}
}

impl serde::Serialize for Measurement {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		let mut state = serializer.serialize_struct("Measurement", 2)?;
		state.serialize_field("time", &self.time)?;
		state.serialize_field("data", &String::from_utf8_lossy(&self.data))?;
		state.end()
	}
}
