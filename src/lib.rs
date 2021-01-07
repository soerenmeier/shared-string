
//! Split a string without another allocation
//!
//! Helpfull for some types that need to be parsed from a string
//! and get split into smaller parts like an `Url` or a `Vec` containing lines
//! which need to be owned by the parent type.
//!
//! ## Note
//!
//! First try to store references, for example `&str` which is more efficient.
//!
//! At the moment if you create a `SharedString` the underlying bytes cannot be
//! mutated.
//!
//! ## Example
//!
//! ```
//! use shared_string::SharedString;
//! // or SharedSyncString if `Sync` is required
//!
//! struct Name {
//! 	firstname: SharedString,
//! 	middlename: SharedString,
//! 	lastname: SharedString
//! 	// to be faster than string
//! 	// you should use at least 3 fields
//! }
//!
//! impl Name {
//! 	pub fn new(fullname: impl Into<SharedString>) -> Option<Self> {
//! 		let mut split = fullname.into().split(b' ');
//! 		Some(Self {
//! 			firstname: split.next()?,
//! 			middlename: split.next()?,
//! 			lastname: split.next()?
//! 		})
//! 	}
//! }
//!
//! let name = Name::new("Bartholomew Jojo Simpson").unwrap();
//! assert_eq!(name.firstname, "Bartholomew");
//! assert_eq!(name.middlename, "Jojo");
//! assert_eq!(name.lastname, "Simpson");
//! ```
//!
//! ## Performance
//!
//! `SharedString` can increase the perfomance in situations such as the example
//! above by over 30%. See `benches/*` for benchmarks.

pub mod iter;
use iter::{Split, Lines};

use std::{ops, str, cmp, fmt, hash, mem, borrow};
use ops::Bound;
use std::string::FromUtf8Error;

use bytes::Bytes;

/// A `SharedString`, generic over its reference counter.
///
/// Most likely you will only need to interact with the type definitions
/// `SharedString` and `SharedSyncString`.
///
/// This struct is useful for parsers or other struct that hold a lot of strings
/// which could all reference. For example a `Uri` Struct or
/// if you have a string with many lines and need every line `independently`.
///
/// ## Lines example
///
/// ```
/// use shared_string::SharedString;
/// // or SharedSyncString if `Sync` is required
///
/// let lines: Vec<_> = SharedString::from("many\nlines\nmany").lines().collect();
/// assert_eq!(lines[0], SharedString::from("many"));
/// assert_eq!(lines[1], "lines");
/// assert_eq!(lines.len(), 3);
/// ```
#[derive(Clone)]
pub struct SharedString(Bytes);

impl SharedString {
	/// Creates a new empty `SharedString`.
	///
	/// This will not allocate.
	#[inline]
	pub const fn new() -> Self {
		Self(Bytes::new())
	}

	// pub fn from_static(s: &'static str) -> 

	#[inline]
	pub unsafe fn from_bytes_unchecked(bytes: Bytes) -> Self {
		Self(bytes)
	}

	/// Convert a vector of bytes to a `SharedString`.
	///
	/// Behaves the same way as [String::from_utf8](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8).
	///
	/// If you are sure that the bytes are valid UTF-8, there is an unsafe
	/// method [from_utf8_unchecked](#method.from_utf8_unchecked) which behaves
	/// the same way but skips the checks.
	///
	/// ## Errors
	///
	/// Returns an `FromUtf8Error` if the bytes are not valid UTF-8 with a
	/// description were an invalid byte was found.
	#[inline]
	pub fn from_utf8(vec: Vec<u8>) -> Result<Self, FromUtf8Error> {
		String::from_utf8(vec).map(|s| s.into())
	}

	/// Converts a vector of bytes to a `SharedString` with out checking that
	/// every bytes is valid UTF-8.
	///
	/// Safe version [from_utf8](#method.from_utf8)
	#[inline]
	pub unsafe fn from_utf8_unchecked(vec: Vec<u8>) -> Self {
		Self(Bytes::from(vec))
	}

	/// Returns a byte slice of the underlying bytes.
	///
	/// To get the full bytes from which this `SharedString` was created from
	/// use [as_bytes_full](#method.as_bytes_full).
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		&*self.0
	}

	/// Returns a string slice of the `SharedString`.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let s = SharedString::from("foo");
	///
	/// assert_eq!("foo", s.as_str());
	/// ```
	#[inline]
	pub fn as_str(&self) -> &str {
		&self
	}

	/// Returns the len of `SharedString`.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let mut foo = SharedString::from("foobar");
	/// let bar = foo.split_off(3);
	///
	/// assert_eq!(3, foo.len());
	/// assert_eq!(3, bar.len());
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Returns `true` if the length is zero, and `false` otherwise.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	// returns new start and end if it is a valid range
	// will be equal to x..y
	// valid: start <= end && end <= len
	#[inline]
	fn validate_range<R>(&self, range: R) -> Option<(usize, usize)>
	where R: ops::RangeBounds<usize> {

		let len = self.len();

		let start = match range.start_bound() {
			Bound::Included(&i) => i,
			Bound::Excluded(&i) => i + 1,
			Bound::Unbounded => 0
		};

		let end = match range.end_bound() {
			Bound::Included(&i) => i + 1,
			Bound::Excluded(&i) => i,
			Bound::Unbounded => len
		};

		if start > end || end > len {
			None
		} else {
			Some((start, end))
		}
	}

	/// Returns a substring of `SharedString`.
	///
	/// This is the non-panicking alternative to [idx](#method.idx) and returns
	/// `None` if the range is out-of-bounds or if the start or the end are not
	/// at a char_boundary.
	///
	/// No allocation is performed.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// # fn inner() -> Option<()> {
	/// let foobar = SharedString::from("foobar");
	///
	/// assert_eq!("foo", foobar.get(..3)?);
	/// assert_eq!("foob", foobar.get(..=3)?);
	/// assert_eq!("foobar", foobar.get(..)?);
	/// assert_eq!("bar", foobar.get(3..)?);
	/// # None
	/// # }
	/// # inner();
	/// ```
	#[inline]
	pub fn get<R>(&self, range: R) -> Option<Self>
	where R: ops::RangeBounds<usize> {
		let (start, end) = self.validate_range(range)?;

		if start == end {
			return Some(Self::new())
		}

		// should validate if is char boundary
		let s = self.as_str();
		if !(s.is_char_boundary(start) && s.is_char_boundary(end)) {
			return None;
		}

		Some(Self(
			self.0.slice(start..end)
		))
	}

	/// Returns a substring of `SharedString` for which no allocation is
	/// performed.
	///
	/// ## Panics
	///
	/// Panics if the range is out-of-bounds.
	///
	/// ## Warning
	///
	/// This method can lead to invalid utf8 if not "split" at a char boundary.
	///
	/// ## Note
	///
	/// The [Index](https://doc.rust-lang.org/std/ops/trait.Index.html) Trait
	/// is not implemented because `index` always returns a reference
	/// and here you always receive an owned type.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let foobar = SharedString::from("foobar");
	///
	/// assert_eq!("foo", foobar.idx(..3));
	/// assert_eq!("foob", foobar.idx(..=3));
	/// assert_eq!("foobar", foobar.idx(..));
	/// assert_eq!("bar", foobar.idx(3..));
	/// ```
	///
	/// ## Todo
	///
	/// Replace trait `AsRange` with
	/// [RangeBounds](https://doc.rust-lang.org/std/ops/trait.RangeBounds.html)
	#[inline]
	pub fn idx<R>(&self, range: R) -> Self
	where R: ops::RangeBounds<usize> {
		Self(self.0.slice(range))
	}

	/// Convert `SharedString` to a `Bytes` instance.
	#[inline]
	pub fn into_bytes(self) -> Bytes {
		self.0
	}

	/// Convert `SharedString` to a `Vec<u8>`.
	///
	/// Copies the underlying data.
	#[inline]
	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	/// Convert `SharedString` to a `String`.
	///
	/// Copies the underlying data.
	#[inline]
	pub fn into_string(self) -> String {
		// Safe because we know the bytes are valid UTF-8
		unsafe { String::from_utf8_unchecked(self.to_vec()) }
	}

	/// Splits the `SharedString` into two at the given index.
	///
	/// No allocation is needed.
	///
	/// This is `O(1)` because only the reference counter is increased.
	///
	/// ## Panics
	///
	/// Panics if `at` is not at a char boundary.
	#[inline]
	pub fn split_off(&mut self, at: usize) -> Self {
		if at == 0 {
			return mem::replace(self, Self::new())
		}

		// panics if at > self.len
		assert!(self.is_char_boundary(at), "not at a char boundary");

		Self(self.0.split_off(at))
	}

	/// Returns an iterator which returns for every "segment" a `SharedString`.
	///
	/// At the moment only u8 as "splitter" is supported.
	///
	/// u8 will be replaced when [Pattern](https://doc.rust-lang.org/std/str/pattern/trait.Pattern.html) gets stabilized.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let mut foobar = SharedString::from("foo bar").split(b' ');
	/// let foo = foobar.next().unwrap();
	/// let bar = foobar.next().unwrap();
	///
	/// assert_eq!(foo, "foo");
	/// assert_eq!(bar, "bar");
	/// ```
	#[inline]
	pub fn split(self, byte: u8) -> Split {
		Split::new(self.0, byte)
	}

	/// Returns an iterator which returns for every line a `SharedString`.
	///
	/// Be aware that this doens't behave exactly like [lines](#method.lines).
	///
	/// This implementation returns an empty line at the if there is a `\n`
	/// byte.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let mut lines = SharedString::from("foo\r\nbar\n\nbaz\n").lines();
	///
	/// assert_eq!("foo", lines.next().unwrap());
	/// assert_eq!("bar", lines.next().unwrap());
	/// assert_eq!("", lines.next().unwrap());
	/// assert_eq!("baz", lines.next().unwrap());
	///
	/// assert_eq!(None, lines.next());
	/// ```
	#[inline]
	pub fn lines(self) -> Lines {
		Lines::new(self.0)
	}

	/// Shortens this `SharedString` to the specified length.
	///
	/// If `new_len` is greater than the current length, nothing happens.
	///
	/// ## Panics
	///
	/// Panics if `new_len` does not lie on a char boundary.
	#[inline]
	pub fn truncate(&mut self, new_len: usize) {
		self.0.truncate(new_len)
	}
}

impl fmt::Display for SharedString {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.as_str(), f)
	}
}

impl fmt::Debug for SharedString {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.as_str(), f)
	}
}

impl hash::Hash for SharedString {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.0.hash(hasher)
	}
}

impl ops::Deref for SharedString {
	type Target = str;

	#[inline]
	fn deref(&self) -> &str {
		// Safe because we know that Self contains valid utf8
		unsafe { str::from_utf8_unchecked(self.as_bytes()) }
	}
}

impl AsRef<str> for SharedString {
	#[inline]
	fn as_ref(&self) -> &str {
		self
	}
}

impl borrow::Borrow<str> for SharedString {
	#[inline]
	fn borrow(&self) -> &str {
		self
	}
}

// need a custom eq
impl cmp::PartialEq for SharedString {
	#[inline]
	fn eq(&self, other: &SharedString) -> bool {
		self.0 == other.0
	}
}

impl cmp::Eq for SharedString {}

impl cmp::PartialEq<str> for SharedString {
	#[inline]
	fn eq(&self, other: &str) -> bool {
		self.0 == other
	}
}

impl<T: ?Sized> cmp::PartialEq<&T> for SharedString
where SharedString: PartialEq<T> {
	#[inline]
	fn eq(&self, other: &&T) -> bool {
		self == *other
	}
}

impl cmp::PartialEq<SharedString> for str {
	#[inline]
	fn eq(&self, other: &SharedString) -> bool {
		self == other.as_str()
	}
}

impl cmp::PartialEq<SharedString> for &str {
	#[inline]
	fn eq(&self, other: &SharedString) -> bool {
		*self == other.0
	}
}


// TODO add a custom ord

impl From<String> for SharedString {
	#[inline]
	fn from(s: String) -> Self {
		Self(s.into())
	}
}

impl From<&'static str> for SharedString {
	#[inline]
	fn from(s: &'static str) -> Self {
		Self(s.into())
	}
}

// Tests
#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn local() {
		let mut hello: SharedString = "Hello, World!".into();
		assert_eq!(hello.len(), 13);

		let world = hello.split_off(7);
		assert_eq!(hello, "Hello, ");
		assert_eq!(hello.len(), 7);
		assert_eq!(world, "World!");
		assert_eq!(world.len(), 6);
	}

	#[test]
	fn in_thread() {
		let mut hello: SharedString = "Hello, World!".into();

		let world = hello.split_off(7);

		std::thread::spawn(move || {
			assert_eq!(world, "World!");
		});

		assert_eq!(hello, "Hello, ");
	}

	#[test]
	fn into() {
		let hello = SharedString::from("Hello, World!");
		let s = hello.into_string();
		assert_eq!(s, "Hello, World!");

		let mut hello: SharedString = s.into();
		let world = hello.split_off(7);

		let s = world.into_string();
		assert_eq!(s, "World!");

		assert!(hello != s.as_str());

		let n_hello = SharedString::from("Hello, ");
		assert_eq!(hello, n_hello);
	}

	#[test]
	fn split_off_zero() {
		let mut foobar = SharedString::from("foobar");
		let n_foobar = foobar.split_off(0);
		assert_eq!("", foobar);
		assert_eq!("foobar", n_foobar);
	}

	#[test]
	#[should_panic]
	fn panic_char_boundary() {
		let mut s = SharedString::from("abc 好 def");
		let _ = s.split_off(5);
	}

	#[test]
	#[should_panic]
	fn panic_length() {
		let mut s = SharedString::from("abc");
		let _ = s.split_off(5);
	}

	#[test]
	fn range_as_str() {
		let raw = SharedString::from("Hello, World!");
		let hello = &raw[..5];
		let world = &raw[7..];

		assert_eq!(hello, "Hello");
		assert_eq!(world, "World!");
	}

	#[test]
	fn range_with_get() {
		let raw = SharedString::from("Hello, World!");
		let hello = raw.get(..5).unwrap();
		let world = raw.get(7..).unwrap();

		assert_eq!(hello, "Hello");
		assert_eq!(world, "World!");
	}

	#[test]
	fn range_with_idx() {
		let raw = SharedString::from("Hello, World!");
		let hello = raw.idx(..5);
		let world = raw.idx(7..);

		assert_eq!(hello, "Hello");
		assert_eq!(world, "World!");
	}

	#[test]
	fn empty() {
		let s = SharedString::from("");
		assert_eq!(s.len(), 0);
		assert!(s.is_empty());

		assert!(s.get(..).unwrap().is_empty());
		assert!(s.get(1..).is_none());
	}

	#[test]
	fn split() {
		let fullname = SharedString::from("Albert Einstein");
		let mut split = fullname.split(b' ');
		assert_eq!(split.next().unwrap(), "Albert");
		assert_eq!(split.next().unwrap(), "Einstein");
		assert_eq!(split.next(), None);
	}

	#[test]
	fn lines() {
		let quote = SharedString::from("Wenn die Menschen nur über das sprächen,\nwas sie begreifen,\r\ndann würde es sehr still auf der Welt sein.\n\r\n");
		let mut lines = quote.lines();
		assert_eq!(
			lines.next().unwrap(),
			"Wenn die Menschen nur über das sprächen,"
		);
		assert_eq!(lines.next().unwrap(), "was sie begreifen,");
		assert_eq!(
			lines.next().unwrap(),
			"dann würde es sehr still auf der Welt sein."
		);
		assert_eq!(lines.next().unwrap(), "");
		assert_eq!(lines.next(), None);

		let empty = SharedString::from(" ");
		let mut lines = empty.lines();
		assert_eq!(" ", lines.next().unwrap());
		assert_eq!(lines.next(), None);
	}

	#[test]
	fn range_eq_str_range() {
		let line = "foo: bar";
		let at = line.find(':').unwrap();
		let key = &line[..at];
		let value = &line[(at + 2)..];

		assert_eq!(key, "foo");
		assert_eq!(value, "bar");

		let line = SharedString::from(line);
		let key = line.idx(..at);
		let value = line.idx((at + 2)..);

		assert_eq!(key, "foo");
		assert_eq!(value, "bar");
	}

	#[test]
	fn range_in_range() {
		let line = "date: Mon, 30 Nov 2020 22:16:22 GMT\nserver: mw1271.eqiad.wmnet\nx-content-type-options: nosniff";
		let mut lines = SharedString::from(line).lines();

		let _ = lines.next().unwrap();
		let line = lines.next().unwrap();

		let at = line.find(':').unwrap();
		assert_eq!(at, 6);

		let key = line.idx(..at);
		assert_eq!(key, "server");

		let value = line.idx((at + 2)..);
		assert_eq!(value, "mw1271.eqiad.wmnet");
	}

	#[test]
	fn truncate() {
		let mut foobar = SharedString::from("foobar");
		foobar.truncate(3);
		assert_eq!(foobar, "foo");
	}
}
