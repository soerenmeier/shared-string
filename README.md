Split a string without another allocation

Helpfull for some types that need to be parsed from a string
and get split into smaller parts like an `Url` or a `Vec` containing lines
which need to be owned by the parent type.

## Note

First try to store references, for example `&str` which is more efficient.

## Example

```rust
use shared_string::SharedString;
// or SharedSyncString if `Sync` is required

struct Name {
	firstname: SharedString,
	lastname: SharedString
}

impl Name {
	pub fn new( fullname: impl Into<SharedString> ) -> Option<Self> {
		let mut split = fullname.into().split(b' ');
		Some(Self {
			firstname: split.next()?,
			lastname: split.next()?
		})
	}
}

let name = Name::new("Albert Einstein").unwrap();
assert_eq!( name.firstname, "Albert" );
assert_eq!( name.lastname, "Einstein" );
```

## Performance

`SharedString` can increase the perfomance in certain cases up to 20% or more.
See `benches/benchmark.rs` for benchmarks.