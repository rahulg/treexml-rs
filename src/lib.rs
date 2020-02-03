//! An element-tree style XML library
//!
//! # Examples
//!
//! ## Reading
//!
//! ```
//! use treexml::Document;
//!
//! let doc_raw = r#"
//! <?xml version="1.1" encoding="UTF-8"?>
//! <table>
//!     <fruit type="apple">worm</fruit>
//!     <vegetable />
//! </table>
//! "#;
//!
//! let doc = Document::parse(doc_raw.as_bytes()).unwrap();
//! let root = doc.root.unwrap();
//!
//! let fruit = root.find_child(|tag| tag.name == "fruit").unwrap().clone();
//! println!("{} [{:?}] = {}", fruit.name, fruit.attributes, fruit.text.unwrap());
//! ```
//!
//! ## Writing
//!
//! ```
//! use treexml::{Document, Element};
//!
//! let mut root = Element::new("root");
//! let mut child = Element::new("child");
//! child.text = Some("contents".to_owned());
//! root.children.push(child);
//!
//! let doc = Document{
//!     root: Some(root),
//!     .. Document::default()
//! };
//!
//! println!("{}", doc);
//! ```
//!
//!

mod builder;
mod document;
mod element;
mod errors;
mod version;

pub use builder::*;
pub use document::Document;
pub use element::Element;
pub use errors::TreexmlError;
pub use version::XmlVersion;
