
//! Iterator types

use crate::SharedString;

use std::mem;

use bytes::{Bytes, Buf};

/// A Split iterator returned by
/// [split](../struct.SharedString.html#method.split).
#[derive(Debug, Clone)]
pub struct Split {
	bytes: Bytes,
	byte: u8
}

impl Split {
	pub(crate) fn new(bytes: Bytes, byte: u8) -> Self {
		Self { bytes, byte }
	}

	// returns index of new byte or self.len
	#[inline]
	fn find_next(&self) -> Option<usize> {
		self.bytes
			.iter()
			.position(|b| b == &self.byte)
	}
}

impl Iterator for Split {
	type Item = SharedString;

	fn next(&mut self) -> Option<Self::Item> {
		if self.bytes.is_empty() {
			return None
		}

		let n_bytes = match self.find_next() {
			Some(p) => {
				let bytes = self.bytes.split_to(p);
				self.bytes.advance(1);
				bytes
			},
			None => mem::take(&mut self.bytes)
		};

		// safe because new can only get called from
		// SharedString
		Some(unsafe {
			SharedString::from_bytes_unchecked(n_bytes)
		})
	}
}

/// A Lines iterator returned by
/// [lines](../struct.SharedString.html#method.lines).
#[derive(Debug, Clone)]
pub struct Lines {
	bytes: Bytes
}

impl Lines {
	pub(crate) fn new(bytes: Bytes) -> Self {
		Self { bytes }
	}

	// returns index of new byte or self.len
	#[inline]
	fn find_next(&self) -> Option<usize> {
		self.bytes
			.iter()
			.position(|&b| b == b'\n')
	}
}

impl Iterator for Lines {
	type Item = SharedString;

	fn next(&mut self) -> Option<Self::Item> {

		if self.bytes.is_empty() {
			return None
		}

		let n_bytes = match self.find_next() {
			Some(p) => {
				let mut bytes = self.bytes.split_to(p);
				self.bytes.advance(1);
				if bytes.ends_with(&[b'\r']) {
					bytes.truncate(bytes.len() - 1);
				}
				bytes
			},
			None => mem::take(&mut self.bytes)
		};

		// safe because new can only get called from
		// SharedString
		Some(unsafe {
			SharedString::from_bytes_unchecked(n_bytes)
		})
	}
}
