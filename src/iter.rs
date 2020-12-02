
//! Iterator types

use crate::{SharedGenString, RefCounter};

/// A Split iterator returned by
/// [split](../struct.SharedGenString.html#method.split).
#[derive(Debug, Clone)]
pub struct Split<R> {
	start: usize,
	end: usize,
	bytes: R,
	byte: u8
}

impl<R> Split<R>
where R: RefCounter {
	pub(crate) fn new(pos: (usize, usize), bytes: R, byte: u8) -> Self {
		Self {
			start: pos.0,
			end: pos.1,
			bytes,
			byte
		}
	}

	#[inline]
	fn slice(&self) -> &[u8] {
		debug_assert!(self.end <= self.bytes.len());
		unsafe { self.bytes.get_unchecked(self.start..=self.end) }
	}

	#[inline]
	fn find_next(&self) -> usize {
		self.slice()
			.iter()
			.position(|b| b == &self.byte)
			.unwrap_or_else(|| (self.end - self.start) + 1)
	}
}

impl<R> Iterator for Split<R>
where R: RefCounter {
	type Item = SharedGenString<R>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.start > self.end {
			return None;
		}

		let start = self.start;
		let at = self.start + self.find_next();
		self.start = at + 1; // to skip byte
		let end = at.saturating_sub(1);

		Some(SharedGenString::new_raw(
			(start, end),
			self.bytes.clone()
		))
	}
}

/// A Lines iterator returned by
/// [lines](../struct.SharedGenString.html#method.lines).
#[derive(Debug, Clone)]
pub struct Lines<R> {
	start: usize,
	end: usize,
	bytes: R
}

impl<R> Lines<R>
where R: RefCounter {
	pub(crate) fn new(pos: (usize, usize), bytes: R) -> Self {
		Self {
			start: pos.0,
			end: pos.1,
			bytes
		}
	}

	#[inline]
	fn slice(&self) -> &[u8] {
		unsafe { self.bytes.get_unchecked(self.start..=self.end) }
	}

	#[inline]
	fn find_next(&self) -> usize {
		self.slice()
			.iter()
			.position(|b| b == &b'\n')
			.unwrap_or_else(|| (self.end - self.start) + 1)
	}
}

impl<R> Iterator for Lines<R>
where R: RefCounter {
	type Item = SharedGenString<R>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.start > self.end {
			return None;
		}

		let start = self.start;
		let next = self.find_next();
		let at = self.start + next;
		self.start = at + 1; // skip \n
		let mut end = at.saturating_sub(1);
		if next >= 1 && self.bytes[end] == b'\r' {
			end -= 1;
		}

		Some(SharedGenString::new_raw(
			(start, end),
			self.bytes.clone()
		))
	}
}
