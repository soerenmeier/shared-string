
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

#[doc(hidden)]
pub mod as_range;
use as_range::AsRange;

pub mod iter;
use iter::{Split, Lines};

use std::{ops, str, cmp, fmt, hash, borrow};
use std::rc::Rc;
use std::sync::Arc;
use std::string::FromUtf8Error;

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
pub struct SharedGenString<R>
where R: RefCounter {
	// maybe replace start with a pointer??
	start: usize,
	len: usize,

	// can only be generated from valid utf8
	bytes: R
}

/// Use `SharedString` if you only need this type in one thread
pub type SharedString = SharedGenString<Rc<Box<[u8]>>>;
/// Use `SharedSyncString` if you need to pass it between threads
pub type SharedSyncString = SharedGenString<Arc<Box<[u8]>>>;

/// A trait to allow `SharedString` to be generic over any reference counter.
///
/// Implemented for `Rc` and `Arc`.
///
/// Requires the traits `Clone` + `Sized` +
/// `Deref<Box<[u8]>>` + `From<Box<[u8]>>`
pub trait RefCounter: Clone + Sized + ops::Deref<Target = Box<[u8]>> + From<Box<[u8]>> {
	fn try_unwrap(self) -> Result<Box<[u8]>, Self>;
}

impl RefCounter for Rc<Box<[u8]>> {
	#[inline]
	fn try_unwrap(self) -> Result<Box<[u8]>, Self> {
		Rc::try_unwrap(self)
	}
}

impl RefCounter for Arc<Box<[u8]>> {
	#[inline]
	fn try_unwrap(self) -> Result<Box<[u8]>, Self> {
		Arc::try_unwrap(self)
	}
}

impl<R> SharedGenString<R>
where R: RefCounter {
	/// Creates a new `SharedString` with the content of `String`.
	///
	/// This will convert the String into a Boxed [u8] slice.
	#[inline]
	pub fn new(string: String) -> Self {
		string.into()
	}

	#[inline]
	pub(crate) fn new_raw(start: usize, len: usize, bytes: R) -> Self {
		Self { start, len, bytes }
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
		Self {
			start: 0,
			len: vec.len(),
			bytes: vec.into_boxed_slice().into()
		}
	}

	/// Returns a byte slice of the underlying bytes.
	///
	/// To get the full bytes from which this `SharedString` was created from
	/// use [as_bytes_full](#method.as_bytes_full).
	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		let end = self.start + self.len;
		unsafe {
			// Safe because we control start and end
			// and know that it is not out-of-bounds
			self.bytes.get_unchecked(self.start..end)
		}
	}

	/// Return a byte slice of the bytes from which this `SharedString` was
	/// created.
	#[inline]
	pub fn as_full_bytes(&self) -> &[u8] {
		&self.bytes
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

	/// Return a string slice of the bytes from which this `SharedString` was
	/// created.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let mut foo = SharedString::from("foobar");
	/// let bar = foo.split_off(3);
	///
	/// assert_eq!("foo", foo.as_str());
	/// assert_eq!("foobar", foo.as_full_str());
	/// ```
	#[inline]
	pub fn as_full_str(&self) -> &str {
		unsafe { str::from_utf8_unchecked(&self.bytes) }
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
		self.len
	}

	/// Returns `true` if the length is zero, and `false` otherwise.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	// returns new start and length if it is a valid range
	#[inline]
	fn validate_range<I>(&self, range: I) -> Option<(usize, usize)>
	where I: AsRange {

		let (mut n_start, n_end) = range.as_range(self.len);
		// if it is a reverse range or a range with len 0
		if n_start >= n_end {
			return None
		}

		let n_len = n_end - n_start;
		n_start += self.start; // add offset

		// check that new range is not out-of-bounds
		if n_start + n_len > self.bytes.len() {
			None
		} else {
			Some((n_start, n_len))
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
	pub fn get<I>(&self, range: I) -> Option<Self>
	where I: AsRange {
		let (n_start, n_len) = self.validate_range(range)?;

		// should validate if is char boundary
		let s = self.as_full_str();
		if !s.is_char_boundary(n_start)
			|| !s.is_char_boundary(n_start + n_len) {
			return None;
		}

		Some(Self {
			start: n_start,
			len: n_len,
			bytes: self.bytes.clone()
		})
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
	/// and here you always received an owned type.
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
	pub fn idx<I>(&self, range: I) -> Self
	where I: AsRange {
		let (n_start, n_len) = self.validate_range(range).unwrap();

		Self {
			start: n_start,
			len: n_len,
			bytes: self.bytes.clone()
		}
	}

	/// Convert `SharedString` to a `Vec<u8>`.
	///
	/// Avoids an allocation if the underlying data is not used by another
	/// instance of `SharedString` and start is at zero.
	#[inline]
	pub fn into_bytes(self) -> Vec<u8> {
		match self.bytes.try_unwrap().map(|b| b.into_vec()) {
			// don't allocate
			Ok(mut bytes) if self.start == 0 => {
				bytes.truncate(self.len);
				bytes
			},
			// needs an allocation
			// Safe because only we control self.start and self.end
			Ok(bytes) => unsafe {
				let range = self.start..(self.start + self.len);
				bytes.get_unchecked(range).to_vec()
			},
			// needs an allocation
			// Safe because only we control self.start and self.end
			Err(slice) => unsafe {
				let range = self.start..(self.start + self.len);
				slice.get_unchecked(range).to_vec()
			}
		}
	}

	/// Returns the underlying Bytes from which this `SharedString` was created.
	///
	/// Tries to avoid a call to `clone` if the underlying data is not used
	/// by another instance.
	#[inline]
	pub fn into_full_bytes(self) -> Vec<u8> {
		match self.bytes.try_unwrap() {
			Ok(bytes) => bytes.into(),
			Err(slice) => slice.to_vec()
		}
	}

	/// Convert `SharedString` to a `String`.
	///
	/// Tries to avoid a call to `clone` if the underlying data is not used
	/// by another instance of `SharedString` and start is at zero.
	#[inline]
	pub fn into_string(self) -> String {
		let vec = self.into_bytes();
		// Safe because we know the bytes are valid UTF-8
		unsafe { String::from_utf8_unchecked(vec) }
	}

	/// Returns the underlying Bytes as a `String` from which this
	/// `SharedString` was created.
	///
	/// Tries to avoid a call to `clone` if the underlying data is not used
	/// by another instance.
	#[inline]
	pub fn into_full_string(self) -> String {
		let vec = self.into_full_bytes();
		unsafe { String::from_utf8_unchecked(vec) }
	}

	/// Pushes a char to the `String` returned by
	/// [into_string](#method.into_string).
	///
	/// If the conditions in `into_string` are met no `clone` is perfomed.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let fooba = SharedString::from("fooba");
	/// let foobar = fooba.push('r');
	///
	/// assert_eq!(foobar, "foobar");
	/// ```
	#[inline]
	pub fn push(self, ch: char) -> String {
		let mut s = self.into_string();
		s.push(ch);
		s
	}

	/// Pushes a string slice to the `String` returned by
	/// [into_string](#method.into_string).
	///
	/// If the conditions in `into_string` are met no `clone` is perfomed.
	///
	/// ## Example
	///
	/// ```
	/// # use shared_string::SharedString;
	/// let foo = SharedString::from("foo");
	/// let foobar = foo.push_str("bar");
	///
	/// assert_eq!(foobar, "foobar");
	/// ```
	#[inline]
	pub fn push_str(self, string: &str) -> String {
		let mut s = self.into_string();
		s.push_str(string);
		s
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
			let c = self.clone();
			self.len = 0;
			return c
		}

		// panics if at > self.len
		assert!(self.is_char_boundary(at));

		let n_len = self.len - at;
		self.len = at;

		Self {
			start: self.start + at,
			len: n_len,
			bytes: self.bytes.clone()
		}
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
	pub fn split(self, byte: u8) -> Split<R> {
		Split::new(self.start, self.len, self.bytes, byte)
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
	pub fn lines(self) -> Lines<R> {
		Lines::new(self.start, self.len, self.bytes)
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
		if new_len < self.len {
			assert!(self.is_char_boundary(new_len));
			self.len = new_len;
		}
	}
}

impl<R> fmt::Display for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.as_str(), f)
	}
}

impl<R> fmt::Debug for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.as_str(), f)
	}
}

impl<R> hash::Hash for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
		self.as_str().hash(hasher)
	}
}

impl<R> ops::Deref for SharedGenString<R>
where R: RefCounter {
	type Target = str;

	#[inline]
	fn deref(&self) -> &str {
		// Safe because we know that Self contains valid utf8
		unsafe { str::from_utf8_unchecked(self.as_bytes()) }
	}
}

impl<R> AsRef<str> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn as_ref(&self) -> &str {
		self
	}
}

impl<R> borrow::Borrow<str> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn borrow(&self) -> &str {
		self
	}
}

// need a custom eq
impl<R, O> cmp::PartialEq<SharedGenString<O>> for SharedGenString<R>
where
	R: RefCounter,
	O: RefCounter {
	#[inline]
	fn eq(&self, other: &SharedGenString<O>) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl<R: RefCounter> cmp::Eq for SharedGenString<R> {}

impl<R> cmp::PartialEq<str> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl<R> cmp::PartialEq<&str> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
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
		*self == other.as_str()
	}
}

impl cmp::PartialEq<SharedSyncString> for str {
	#[inline]
	fn eq(&self, other: &SharedSyncString) -> bool {
		self == other.as_str()
	}
}

impl cmp::PartialEq<SharedSyncString> for &str {
	#[inline]
	fn eq(&self, other: &SharedSyncString) -> bool {
		*self == other.as_str()
	}
}

// need a custom ord

impl<R> From<String> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn from(s: String) -> Self {
		Self {
			start: 0,
			len: s.len(),
			bytes: s.into_bytes().into_boxed_slice().into()
		}
	}
}

impl<R> From<&str> for SharedGenString<R>
where R: RefCounter {
	#[inline]
	fn from(s: &str) -> Self {
		s.to_string().into()
	}
}

// Tests
#[cfg(test)]
mod tests {

	use super::{SharedString, SharedSyncString};

	#[test]
	fn rc() {
		let mut hello: SharedString = "Hello, World!".into();
		assert_eq!(hello.len(), 13);

		let world = hello.split_off(7);
		assert_eq!(hello, "Hello, ");
		assert_eq!(hello.len(), 7);
		assert_eq!(world, "World!");
		assert_eq!(world.len(), 6);
	}

	#[test]
	fn arc() {
		let mut hello: SharedSyncString = "Hello, World!".into();

		let world = hello.split_off(7);

		std::thread::spawn(move || {
			assert_eq!(world, "World!");
			assert_eq!(world.as_full_str(), "Hello, World!");
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

		assert!(s.get(..).is_none());
		assert!(s.get(1..).is_none());
	}

	#[test]
	fn equal() {
		let rc: SharedString = "Hello, World!".into();
		let arc: SharedSyncString = "Hello, World!".into();
		assert_eq!(rc, arc);
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
