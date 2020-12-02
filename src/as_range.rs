
use std::ops;

mod private {
	use std::ops;

	pub trait Sealed {}

	impl Sealed for ops::Range<usize> {}
	impl Sealed for ops::RangeFrom<usize> {}
	impl Sealed for ops::RangeFull {}
	impl Sealed for ops::RangeInclusive<usize> {}
	impl Sealed for ops::RangeTo<usize> {}
	impl Sealed for ops::RangeToInclusive<usize> {}
}

pub trait AsRange: private::Sealed {
	fn as_range(self, len: usize) -> (usize, usize);
}

impl AsRange for ops::Range<usize> {
	#[inline]
	fn as_range(self, _: usize) -> (usize, usize) {
		(self.start, self.end)
	}
}

impl AsRange for ops::RangeFrom<usize> {
	#[inline]
	fn as_range(self, len: usize) -> (usize, usize) {
		(self.start, len)
	}
}

impl AsRange for ops::RangeFull {
	#[inline]
	fn as_range(self, len: usize) -> (usize, usize) {
		(0, len)
	}
}

impl AsRange for ops::RangeInclusive<usize> {
	#[inline]
	fn as_range(self, _: usize) -> (usize, usize) {
		(*self.start(), *self.end() + 1)
	}
}

impl AsRange for ops::RangeTo<usize> {
	#[inline]
	fn as_range(self, _: usize) -> (usize, usize) {
		(0, self.end)
	}
}

impl AsRange for ops::RangeToInclusive<usize> {
	#[inline]
	fn as_range(self, _: usize) -> (usize, usize) {
		(0, self.end + 1)
	}
}

#[cfg(test)]
mod tests {

	use super::AsRange;

	#[test]
	fn as_range() {

		assert_eq!((0, 0), (0..0).as_range(10));
		assert_eq!((0, 2), (0..2).as_range(10));
		assert_eq!((0, 6), (0..=5).as_range(4));
		assert_eq!((0, 5), (0..).as_range(5));
		assert_eq!((0, 5), (..).as_range(5));
		assert_eq!((0, 6), (..6).as_range(5));
		assert_eq!((0, 7), (..=6).as_range(5));

		assert_eq!((6, 2), (6..2).as_range(5));
	}
}
