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

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate failure;

extern crate indexmap;

mod errors;

extern crate xml;

mod builder;

use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};
use std::iter::Filter;
use std::slice::{Iter, IterMut};
use std::str::FromStr;
use std::string::ToString;

pub use errors::*;

pub use builder::*;

use indexmap::IndexMap;

use xml::common::XmlVersion as BaseXmlVersion;

/// Enumeration of XML versions
///
/// This exists solely because `xml-rs`'s `XmlVersion` doesn't implement Debug
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum XmlVersion {
    /// XML Version 1.0
    Version10,
    /// XML Version 1.1
    Version11,
}

impl From<BaseXmlVersion> for XmlVersion {
    fn from(value: BaseXmlVersion) -> XmlVersion {
        match value {
            BaseXmlVersion::Version10 => XmlVersion::Version10,
            BaseXmlVersion::Version11 => XmlVersion::Version11,
        }
    }
}

impl From<XmlVersion> for BaseXmlVersion {
    fn from(value: XmlVersion) -> BaseXmlVersion {
        match value {
            XmlVersion::Version10 => BaseXmlVersion::Version10,
            XmlVersion::Version11 => BaseXmlVersion::Version11,
        }
    }
}

/// An XML element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// Tag prefix, used for namespacing: `xsl` in `xsl:for-each`
    pub prefix: Option<String>,
    /// Tag name: `for-each` in `xsl:for-each`
    pub name: String,
    /// Tag attributes
    pub attributes: IndexMap<String, String>,
    /// A vector of child elements
    pub children: Vec<Element>,
    /// Contents of the element
    pub text: Option<String>,
    /// CDATA contents of the element
    pub cdata: Option<String>,
}

impl Default for Element {
    fn default() -> Self {
        Element {
            prefix: None,
            name: "tag".to_owned(),
            attributes: IndexMap::new(),
            children: Vec::new(),
            text: None,
            cdata: None,
        }
    }
}

impl Element {
    /// Create a new `Element` with the tag name `name`
    pub fn new<S>(name: S) -> Element
    where
        S: ToString,
    {
        Element {
            name: name.to_string(),
            ..Element::default()
        }
    }

    /// Parse the contents of an element
    fn parse<R: Read>(
        &mut self,
        mut reader: &mut xml::reader::EventReader<R>,
    ) -> Result<(), Error> {
        use xml::reader::XmlEvent;

        loop {
            let ev = reader.next()?;
            match ev {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    let mut attr_map = IndexMap::new();
                    for attr in attributes {
                        let attr_name = match attr.name.prefix {
                            Some(prefix) => format!("{}:{}", prefix, attr.name.local_name),
                            None => attr.name.local_name,
                        };
                        attr_map.insert(attr_name, attr.value);
                    }

                    let mut child = Element {
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        ..Element::default()
                    };
                    child.parse(&mut reader)?;
                    self.children.push(child);
                }
                XmlEvent::EndElement { name } => {
                    if name.prefix == self.prefix && name.local_name == self.name {
                        return Ok(());
                    } else {
                        // This should never happen, since the base xml library will panic first
                        panic!("Unexpected closing tag: {}, expected {}", name, self.name);
                    }
                }
                XmlEvent::Characters(s) => {
                    let text = match self.text {
                        Some(ref v) => v.clone(),
                        None => String::new(),
                    };
                    self.text = Some(text + &s);
                }
                XmlEvent::CData(s) => {
                    let cdata = match self.cdata {
                        Some(ref v) => v.clone(),
                        None => String::new(),
                    };
                    self.cdata = Some(cdata + &s);
                }
                XmlEvent::StartDocument { .. }
                | XmlEvent::EndDocument
                | XmlEvent::ProcessingInstruction { .. }
                | XmlEvent::Whitespace(_)
                | XmlEvent::Comment(_) => {}
            }
        }
    }

    /// Write an element and its contents to `writer`
    fn write<W: Write>(&self, writer: &mut xml::writer::EventWriter<W>) -> Result<(), Error> {
        use xml::attribute::Attribute;
        use xml::name::Name;
        use xml::namespace::Namespace;
        use xml::writer::XmlEvent;

        let name = Name::local(&self.name);
        let mut attributes = Vec::with_capacity(self.attributes.len());
        for (k, v) in &self.attributes {
            attributes.push(Attribute {
                name: Name::local(k),
                value: v,
            });
        }

        let namespace = Namespace::empty();

        writer.write(XmlEvent::StartElement {
            name: name,
            attributes: Cow::Owned(attributes),
            namespace: Cow::Owned(namespace),
        })?;

        if let Some(ref text) = self.text {
            writer.write(XmlEvent::Characters(&text[..]))?;
        }
        if let Some(ref cdata) = self.cdata {
            writer.write(XmlEvent::CData(&cdata[..]))?;
        }

        for e in &self.children {
            e.write(writer)?;
        }

        writer.write(XmlEvent::EndElement { name: Some(name) })?;

        Ok(())
    }

    /// Find a single child of the current `Element`, given a predicate
    pub fn find_child<P>(&self, predicate: P) -> Option<&Element>
    where
        P: for<'r> Fn(&'r &Element) -> bool,
    {
        self.children.iter().find(predicate)
    }

    /// Find a single child of the current `Element`, given a predicate; returns a mutable borrow
    pub fn find_child_mut<P>(&mut self, predicate: P) -> Option<&mut Element>
    where
        P: for<'r> FnMut(&'r &mut Element) -> bool,
    {
        self.children.iter_mut().find(predicate)
    }

    /// Traverse element using an xpath-like string: root/child/a
    pub fn find(&self, path: &str) -> Result<&Element, Error> {
        Self::find_path(&path.split('/').collect::<Vec<&str>>(), path, self)
    }

    pub fn find_value<T: FromStr>(&self, path: &str) -> Result<Option<T>, Error> {
        let el = self.find(path)?;
        if let Some(text) = el.text.as_ref() {
            match T::from_str(text) {
                Err(_) => Err(errors::Error::ValueFromStr {
                    t: text.to_string(),
                }.into()),
                Ok(value) => Ok(Some(value)),
            }
        } else {
            Ok(None)
        }
    }

    fn find_path<'a>(
        path: &[&str],
        original: &str,
        tree: &'a Element,
    ) -> Result<&'a Element, Error> {
        if path.is_empty() {
            return Ok(tree);
        }

        match tree.find_child(|t| t.name == path[0]) {
            Some(element) => Self::find_path(&path[1..], original, element),
            None => Err(errors::Error::ElementNotFound { t: original.into() }.into()),
        }
    }

    /// Filters the children of the current `Element`, given a predicate
    pub fn filter_children<P>(&self, predicate: P) -> Filter<Iter<Element>, P>
    where
        P: for<'r> Fn(&'r &Element) -> bool,
    {
        self.children.iter().filter(predicate)
    }

    /// Filters the children of the current `Element`, given a predicate; returns a mutable iterator
    pub fn filter_children_mut<P>(&mut self, predicate: P) -> Filter<IterMut<Element>, P>
    where
        P: for<'r> FnMut(&'r &mut Element) -> bool,
    {
        self.children.iter_mut().filter(predicate)
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let doc = Document {
            root: Some(self.clone()),
            ..Document::default()
        };
        let mut v = Vec::<u8>::new();
        doc.write_with(&mut v, false, "  ", true).unwrap();
        let s = String::from_utf8(v).unwrap();
        f.write_str(&s[..])
    }
}

/// An XML document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Version of the XML document
    pub version: XmlVersion,
    /// Encoding of the XML document
    pub encoding: String,
    /// Root tag of the XML document
    pub root: Option<Element>,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            version: XmlVersion::Version10,
            encoding: "UTF-8".to_owned(),
            root: None,
        }
    }
}

impl Document {
    /// Create a new `Document` with default values
    pub fn new() -> Document {
        Document {
            ..Document::default()
        }
    }

    /// Create a new `Document` with an Element or ElementBuilder at its root.
    pub fn build(root: &mut ElementBuilder) -> Self {
        Document {
            root: Some(root.element()),
            ..Self::default()
        }
    }

    /// Parse data from a reader to construct an XML document
    ///
    /// # Failures
    ///
    /// Passes any errors that the `xml-rs` library returns up the stack
    pub fn parse<R: Read>(r: R) -> Result<Document, Error> {
        use xml::reader::{EventReader, XmlEvent};

        let mut reader = EventReader::new(r);
        let mut doc = Document::new();

        loop {
            let ev = reader.next()?;
            match ev {
                XmlEvent::StartDocument {
                    version, encoding, ..
                } => {
                    doc.version = XmlVersion::from(version);
                    doc.encoding = encoding;
                }
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    // Start of the root element

                    let mut attr_map = IndexMap::new();
                    for attr in attributes {
                        let attr_name = match attr.name.prefix {
                            Some(prefix) => format!("{}:{}", prefix, attr.name.local_name),
                            None => attr.name.local_name,
                        };
                        attr_map.insert(attr_name, attr.value);
                    }

                    let mut root = Element {
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        ..Element::default()
                    };
                    root.parse(&mut reader)?;
                    doc.root = Some(root);
                }
                XmlEvent::EndDocument => break,
                _ => {}
            }
        }

        Ok(doc)
    }

    pub fn write<W: Write>(&self, mut w: &mut W) -> Result<(), Error> {
        self.write_with(&mut w, true, "  ", true)
    }

    /// Writes a document to `w`
    pub fn write_with<W: Write>(
        &self,
        w: &mut W,
        document_decl: bool,
        indent_str: &'static str,
        indent: bool,
    ) -> Result<(), Error> {
        use xml::writer::{EmitterConfig, XmlEvent};

        let mut writer = EmitterConfig::new()
            .perform_indent(indent)
            .write_document_declaration(document_decl)
            .indent_string(indent_str)
            .create_writer(w);

        if document_decl {
            writer.write(XmlEvent::StartDocument {
                version: self.version.into(),
                encoding: Some(&self.encoding),
                standalone: None,
            })?;
        }

        if let Some(ref e) = self.root {
            e.write(&mut writer)?;
        }

        Ok(())
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut v = Vec::<u8>::new();
        self.write(&mut v).unwrap();
        let s = String::from_utf8(v).unwrap();
        f.write_str(&s[..])
    }
}
