
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


pub trait AsRangeInclusive: private::Sealed {
	fn as_range_inclusive( self, end: usize ) -> (usize, usize);
}

impl AsRangeInclusive for ops::Range<usize> {
	#[inline]
	fn as_range_inclusive( self, _: usize ) -> (usize, usize) {
		(self.start, self.end.max(1) - 1)
	}
}

impl AsRangeInclusive for ops::RangeFrom<usize> {
	#[inline]
	fn as_range_inclusive( self, end: usize ) -> (usize, usize) {
		(self.start, end)
	}
}

impl AsRangeInclusive for ops::RangeFull {
	#[inline]
	fn as_range_inclusive( self, end: usize ) -> (usize, usize) {
		(0, end)
	}
}

impl AsRangeInclusive for ops::RangeInclusive<usize> {
	#[inline]
	fn as_range_inclusive( self, _: usize ) -> (usize, usize) {
		(*self.start(), *self.end())
	}
}

impl AsRangeInclusive for ops::RangeTo<usize> {
	#[inline]
	fn as_range_inclusive( self, end: usize ) -> (usize, usize) {
		(0..self.end).as_range_inclusive( end )
	}
}

impl AsRangeInclusive for ops::RangeToInclusive<usize> {
	#[inline]
	fn as_range_inclusive( self, _: usize ) -> (usize, usize) {
		(0, self.end)
	}
}



#[cfg(test)]
mod tests {

	use super::AsRangeInclusive;

	#[test]
	fn as_range_inclusive() {

		assert_eq!( (0, 0), (0..0).as_range_inclusive( 10 ) );
		assert_eq!( (0, 1), (0..2).as_range_inclusive( 10 ) );
		assert_eq!( (0, 5), (0..=5).as_range_inclusive( 4 ) );
		assert_eq!( (0, 5), (0..).as_range_inclusive( 5 ) );
		assert_eq!( (0, 5), (..).as_range_inclusive( 5 ) );
		assert_eq!( (0, 5), (..6).as_range_inclusive( 5 ) );
		assert_eq!( (0, 6), (..=6).as_range_inclusive( 5 ) );

		assert_eq!( (6, 1), (6..2).as_range_inclusive( 5 ) );

	}

}