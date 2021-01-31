// Benchmarking the main example
//
// Using lines instead of split()
// because the current implementation of
// split only supports a byte as argument
// and the std implementation of &str.split()
// can take many different types

use shared_string::SharedString;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

struct NameString {
	pub firstname: String,
	pub middlename: String,
	pub lastname: String
}

impl NameString {
	pub fn new(fullname: &str) -> Option<Self> {
		let mut split = fullname.lines();
		Some(Self {
			firstname: split.next()?.into(),
			middlename: split.next()?.into(),
			lastname: split.next()?.into()
		})
	}
}

struct NameShared {
	pub firstname: SharedString,
	pub middlename: SharedString,
	pub lastname: SharedString
}

impl NameShared {
	pub fn new(fullname: &str) -> Option<Self> {
		let mut split = SharedString::from(String::from(fullname)).lines();
		Some(Self {
			firstname: split.next()?,
			middlename: split.next()?,
			lastname: split.next()?
		})
	}
}

fn benchmark_name(c: &mut Criterion) {
	let raw_name = "Bartholomew\nJojo\nSimpson";

	c.bench_function("name_string", |b| {
		b.iter(|| {
			NameString::new(black_box(raw_name))
				.unwrap()
		})
	});

	c.bench_function("name_shared", |b| {
		b.iter(|| {
			NameShared::new(black_box(raw_name))
				.unwrap()
		})
	});
}

criterion_group!(bench_name, benchmark_name);

criterion_main!(bench_name);