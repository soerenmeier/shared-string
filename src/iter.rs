
//! Iterator types

use crate::{SharedGenString, RefCounter};

/// A Split iterator returned by
/// [split](../struct.SharedGenString.html#method.split).
#[derive(Debug, Clone)]
pub struct Split<R> {
	start: usize,
	len: usize,
	bytes: R,
	byte: u8
}

impl<R> Split<R>
where R: RefCounter {
	pub(crate) fn new(start: usize, len: usize, bytes: R, byte: u8) -> Self {
		Self { start, len, bytes, byte }
	}

	#[inline]
	fn remaning_slice(&self) -> &[u8] {
		// Safe because only we control start and len
		let range = self.start..(self.start + self.len);
		unsafe { self.bytes.get_unchecked(range) }
	}

	// returns index of new byte or self.len
	#[inline]
	fn find_next(&self) -> usize {
		self.remaning_slice()
			.iter()
			.position(|b| b == &self.byte)
			.unwrap_or(self.len)
	}
}

impl<R> Iterator for Split<R>
where R: RefCounter {
	type Item = SharedGenString<R>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.len == 0 {
			return None
		}

		let at = self.find_next();
		let n_at = at + 1; // might out-of-bound

		let n_start = self.start;
		self.start += n_at;
		self.len = self.len.saturating_sub(n_at);
		Some(SharedGenString::new_raw(
			n_start,
			at,
			self.bytes.clone()
		))
	}
}

/// A Lines iterator returned by
/// [lines](../struct.SharedGenString.html#method.lines).
#[derive(Debug, Clone)]
pub struct Lines<R> {
	start: usize,
	len: usize,
	bytes: R
}

impl<R> Lines<R>
where R: RefCounter {
	pub(crate) fn new(start: usize, len: usize, bytes: R) -> Self {
		Self { start, len, bytes }
	}

	#[inline]
	fn remaning_slice(&self) -> &[u8] {
		// Safe because only we control start and len
		let range = self.start..(self.start + self.len);
		unsafe { self.bytes.get_unchecked(range) }
	}

	// returns index of new byte or self.len
	#[inline]
	fn find_next(&self) -> usize {
		self.remaning_slice()
			.iter()
			.position(|&b| b == b'\n')
			.unwrap_or(self.len)
	}
}

impl<R> Iterator for Lines<R>
where R: RefCounter {
	type Item = SharedGenString<R>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.len == 0 {
			return None
		}

		let mut at = self.find_next();
		// + 1 for skipping \n
		let newline_at = at + 1; // could be out-of-bound

		let n_start = self.start;
		self.start += newline_at;
		self.len = self.len.saturating_sub(newline_at);

		// check if should do at - 1 (to remove \r)
		if at >= 1 && self.bytes[n_start + at - 1] == b'\r' {
			at -= 1;
		}

		Some(SharedGenString::new_raw(
			n_start,
			at,
			self.bytes.clone()
		))
	}
}
