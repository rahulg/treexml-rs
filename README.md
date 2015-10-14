# `treexml`: An XML Tree Library for Rust

`treexml` is a simple element-tree style library for XML data.

## Usage

Like most `rust` packages, `treexml` uses [cargo](http://crates.io).
To use `treexml`, add the following to your project's `Cargo.toml`

```toml
[dependencies]
treexml = "0.2"
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

	let doc_raw = r#"
	<?xml version="1.1" encoding="UTF-8"?>
	<table>
		<fruit type="apple">worm</fruit>
		<vegetable />
	</table>
	"#;

	let doc = Document::parse(doc_raw.as_bytes()).unwrap();
	let root = doc.root.unwrap();

	let fruit = root.find_child(|tag| tag.name == "fruit").unwrap().clone();
	println!("{} [{:?}] = {}", fruit.name, fruit.attributes, fruit.contents.unwrap());
	
}
```

## Writing XML Data

This is currently not supported, but is on the cards for a future version.

## Contributing

This project is licensed under the MIT license.

If you encounter any issues, please file them on the GitHub issue tracker at https://github.com/rahulg/treexml/issues.
