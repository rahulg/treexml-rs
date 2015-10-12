# `treexml`: An XML Tree Library for Rust

`treexml` is a simple element-tree style library for XML data.

## Usage

Like most `rust` packages, `treexml` uses [cargo](http://crates.io).
To use `treexml`, add the following to your project's `Cargo.toml`

```toml
[dependencies]
treexml = "0.1"
```

The package exposes a crate named `treexml`.

```rust
extern crate treexml;
```

## Reading XML Data

Assuming `r` is something that implements `std::io::Read`:

```rust
extern crate treexml;

use treexml::Document;

fn main() {

	// ...
	// code that opens a file / fetches data from an API and assigns r
	// ...

	let doc = Document::parse(r).unwrap();

	let elem = match doc.root {
		None => panic!("no data"),
		Some(r) => r.children[0].clone(),
	};

	let contents = match elem.contents {
		None => "".to_owned(),
		Some(s) => s.clone(),
	};

	println!("{} [{:?}] = {}", elem.name, elem.attributes, contents);
	
}
```

## Writing XML Data

This is currently not supported, but is on the cards for a future version.

## Contributing

This project is licensed under the MIT license.

If you encounter any issues, please file them on the GitHub issue tracker at https://github.com/rahulg/treexml/issues.
