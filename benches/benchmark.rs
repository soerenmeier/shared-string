
use std::collections::HashMap;
use std::io::{ Read, BufReader, BufRead };
use std::mem;

use shared_string::{ SharedString, SharedSyncString };

use criterion::{ black_box, criterion_group, criterion_main, Criterion };


// taken from a response from wikipedia.org
const HTTP_HEADER: &'static str = "\
date: Mon, 30 Nov 2020 22:16:22 GMT
server: mw1271.eqiad.wmnet
x-content-type-options: nosniff
p3p: CP=\"See https://de.wikipedia.org/wiki/Special:CentralAutoLogin/P3P for more info.\"
content-language: de
vary: Accept-Encoding,Cookie,Authorization
x-request-id: b4e70d43-5e25-4cad-aea5-1bbaf8f11120
last-modified: Mon, 30 Nov 2020 19:37:27 GMT
content-type: text/html; charset=UTF-8
content-encoding: gzip
age: 55974
x-cache: cp3054 miss, cp3060 hit/15
x-cache-status: hit-front
server-timing: cache;desc=\"hit-front\"
strict-transport-security: max-age=106384710; includeSubDomains; preload
x-client-ip: 194.158.250.88
cache-control: private, s-maxage=0, max-age=0, must-revalidate
X-Firefox-Spdy: h2\
";


// String

fn parse_to_string( string: String ) -> HashMap<String, String> {
	let mut map = HashMap::new();
	for line in string.lines() {
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line[..at].to_string();
		// we can skip the space here because we know after every colon is a space
		let value = line[(at + 2)..].to_string();

		map.insert( key, value );
	}

	map
}

fn parse_to_shared_string( string: String ) -> HashMap<SharedString, SharedString> {
	let string = SharedString::from(string);
	let mut map = HashMap::new();
	for line in string.lines() {
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line.idx(..at);
		// we can skip the space here because we know after every colon is a space
		let value = line.idx((at + 2)..);

		map.insert( key, value );
	}

	map
}

fn parse_to_shared_sync_string( string: String ) -> HashMap<SharedSyncString, SharedSyncString> {
	let string = SharedSyncString::from(string);
	let mut map = HashMap::new();
	for line in string.lines() {
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line.idx(..at);
		// we can skip the space here because we know after every colon is a space
		let value = line.idx((at + 2)..);

		map.insert( key, value );
	}

	map
}

fn benchmark_string( c: &mut Criterion ) {

	c.bench_function("parse_to_string", |b| b.iter( || {
		let http_header = HTTP_HEADER.to_string();
		parse_to_string(black_box( http_header ))
	} ) );

	c.bench_function("parse_to_shared_string", |b| b.iter( || {
		let http_header = HTTP_HEADER.to_string();
		parse_to_shared_string(black_box( http_header ))
	} ) );

	c.bench_function("parse_to_shared_sync_string", |b| b.iter( || {
		let http_header = HTTP_HEADER.to_string();
		parse_to_shared_sync_string(black_box( http_header ))
	} ) );

}

// BufReader

fn parse_to_string_from_buf_reader<T: Read>( mut reader: BufReader<T> ) -> HashMap<String, String> {
	let mut map = HashMap::new();
	let mut line = String::with_capacity(100);

	// this is faster than parse_to_string
	// because read_line does not check for \r\n
	// and doesn't strip them
	while 0 != reader.read_line( &mut line ).unwrap() {
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line[..at].to_string();
		// we can skip the space here because we know after every colon is a space
		let value = line[(at + 2)..].to_string();

		map.insert( key, value );
	}

	map
}

fn parse_to_shared_string_from_buf_reader<T: Read>( mut reader: BufReader<T> ) -> HashMap<SharedString, SharedString> {
	let mut map = HashMap::new();
	let mut line = String::with_capacity(100);

	while 0 != reader.read_line( &mut line ).unwrap() {
		let line: SharedString = line.clone().into();
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line.idx(..at);
		// we can skip the space here because we know after every colon is a space
		let value = line.idx((at + 2)..);

		map.insert( key, value );
	}

	map
}

fn parse_to_shared_string_from_buf_reader_with_split_off<T: Read>( mut reader: BufReader<T> ) -> HashMap<SharedString, SharedString> {
	let mut map = HashMap::new();
	let mut line = String::with_capacity(100);

	while 0 != reader.read_line( &mut line ).unwrap() {
		let mut key: SharedString = line.clone().into();
		// unwrap because we know that in every line is a colon
		let at = key.find(':').unwrap();

		let value = key.split_off(at);
		// we can skip the space here because we know after every colon is a space
		let value = value.idx(2..);

		map.insert( key, value );
	}

	map
}

fn parse_to_shared_sync_string_from_buf_reader<T: Read>( mut reader: BufReader<T> ) -> HashMap<SharedSyncString, SharedSyncString> {
	let mut map = HashMap::new();
	let mut line = String::with_capacity(100);

	while 0 != reader.read_line( &mut line ).unwrap() {
		let line: SharedSyncString = line.clone().into();
		// unwrap because we know that in every line is a colon
		let at = line.find(':').unwrap();

		let key = line.idx(..at);
		// we can skip the space here because we know after every colon is a space
		let value = line.idx((at + 2)..);

		map.insert( key, value );
	}

	map
}

fn benchmark_buf_reader( c: &mut Criterion ) {

	let bytes = HTTP_HEADER.as_bytes().to_vec();

	c.bench_function("parse_to_string_from_buf_reader", |b| b.iter( || {
		let reader = BufReader::new( bytes.as_slice() );
		parse_to_string_from_buf_reader(black_box( reader ))
	} ) );

	c.bench_function("parse_to_shared_string_from_buf_reader", |b| b.iter( || {
		let reader = BufReader::new( bytes.as_slice() );
		parse_to_shared_string_from_buf_reader(black_box( reader ))
	} ) );

	c.bench_function("parse_to_shared_string_from_buf_reader_with_split_off", |b| b.iter( || {
		let reader = BufReader::new( bytes.as_slice() );
		parse_to_shared_string_from_buf_reader_with_split_off(black_box( reader ))
	} ) );

	c.bench_function("parse_to_shared_sync_string_from_buf_reader", |b| b.iter( || {
		let reader = BufReader::new( bytes.as_slice() );
		parse_to_shared_sync_string_from_buf_reader(black_box( reader ))
	} ) );

}

criterion_group!( bench_string, benchmark_string );
criterion_group!( bench_buf_reader, benchmark_buf_reader );

criterion_main!( bench_string, bench_buf_reader );