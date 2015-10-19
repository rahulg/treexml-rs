//! An element-tree style XML library
//!
//! # Examples
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

extern crate xml;

use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::iter::Filter;
use std::slice::{Iter, IterMut};

/// The common error type for all XML tree operations
#[derive(Debug)]
pub enum Error {
    /// Error parsing input data into an XML tree
    ParseError(xml::reader::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError(ref e) => e.description(),
        }
    }
}

impl From<xml::reader::Error> for Error {
    fn from(err: xml::reader::Error) -> Error {
        Error::ParseError(err)
    }
}

/// Enumeration of XML versions
///
/// This exists solely because `xml-rs`'s XmlVersion doesn't implement Debug
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum XmlVersion {
    /// XML Version 1.0
    Version10,
    /// XML Version 1.1
    Version11,
}

impl From<xml::common::XmlVersion> for XmlVersion {
    fn from(v: xml::common::XmlVersion) -> XmlVersion {
        match v {
            xml::common::XmlVersion::Version10 => XmlVersion::Version10,
            xml::common::XmlVersion::Version11 => XmlVersion::Version11,
        }
    }
}

impl Into<xml::common::XmlVersion> for XmlVersion {
    fn into(self) -> xml::common::XmlVersion {
        match self {
            XmlVersion::Version10 => xml::common::XmlVersion::Version10,
            XmlVersion::Version11 => xml::common::XmlVersion::Version11,
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
    pub attributes: HashMap<String, String>,
    /// A vector of child elements
    pub children: Vec<Element>,
    /// Contents of the element
    pub text: Option<String>,
    /// CDATA contents of the element
    pub cdata: Option<String>,
}

impl Default for Element {
    fn default() -> Self {
        Element{
            prefix: None,
            name: "tag".to_owned(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text: None,
            cdata: None,
        }
    }
}

impl Element {

    /// Create a new `Element` with the tag name `name`
    pub fn new<S>(name: S) -> Element
        where S: Into<String>
    {
        Element{name: name.into(), .. Element::default()}
    }

    /// Parse the contents of an element
    fn parse<R: Read>(&mut self, mut reader: &mut xml::reader::EventReader<R>) -> Result<(), Error> {

        use xml::reader::XmlEvent;

        loop {
            let ev = try!(reader.next());
            match ev {
                XmlEvent::StartElement{name, attributes, ..} => {

                    let mut attr_map = HashMap::new();
                    for attr in attributes {
                        let attr_name = match attr.name.prefix {
                            Some(prefix) => format!("{}:{}", prefix, attr.name.local_name),
                            None => attr.name.local_name,
                        };
                        attr_map.insert(attr_name, attr.value);
                    }

                    let mut child = Element{
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        .. Element::default()
                    };
                    try!(child.parse(&mut reader));
                    self.children.push(child);

                },
                XmlEvent::EndElement{name} => {

                    if name.prefix == self.prefix && name.local_name == self.name {
                        return Ok(());
                    } else {
                        // This should never happen, since the base xml library will panic first
                        panic!("Unexpected closing tag: {}, expected {}", name, self.name);
                    }

                },
                XmlEvent::Characters(s) => {

                    let text = match self.text {
                        Some(ref v) => v.clone(),
                        None => String::new(),
                    };
                    self.text = Some(text + &s)

                },
                XmlEvent::CData(s) => {

                    let cdata = match self.cdata {
                        Some(ref v) => v.clone(),
                        None => String::new(),
                    };
                    self.cdata = Some(cdata + &s);

                },
                XmlEvent::Whitespace(_) => {},
                XmlEvent::Comment(_) => {},
                _ => {},
            }
        }
    }

    /// Find a single child of the current `Element`, given a predicate
    pub fn find_child<P>(&self, predicate: P) -> Option<&Element>
        where P: for<'r> Fn(&'r &Element) -> bool
    {
        self.children.iter().find(predicate)
    }

    /// Find a single child of the current `Element`, given a predicate; returns a mutable borrow
    pub fn find_child_mut<P>(&mut self, predicate: P) -> Option<&mut Element>
        where P: for<'r> FnMut(&'r &mut Element) -> bool
    {
        self.children.iter_mut().find(predicate)
    }

    /// Filters the children of the current `Element`, given a predicate
    pub fn filter_children<P>(&self, predicate: P) -> Filter<Iter<Element>, P>
        where P: for<'r> Fn(&'r &Element) -> bool
    {
        self.children.iter().filter(predicate)
    }

    /// Filters the children of the current `Element`, given a predicate; returns a mutable iterator
    pub fn filter_children_mut<P>(&mut self, predicate: P) -> Filter<IterMut<Element>, P>
        where P: for<'r> FnMut(&'r &mut Element) -> bool
    {
        self.children.iter_mut().filter(predicate)
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
        Document{
            version: XmlVersion::Version10,
            encoding: "UTF-8".to_owned(),
            root: None,
        }
    }
}

impl Document {

    /// Create a new `Document` with default values
    pub fn new() -> Document {
        Document{.. Document::default()}
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
            let ev = try!(reader.next());
            match ev {
                XmlEvent::StartDocument{version, encoding, ..} => {
                    doc.version = XmlVersion::from(version);
                    doc.encoding = encoding;
                },
                XmlEvent::StartElement{name, attributes, ..} => {

                    // Start of the root element

                    let mut attr_map = HashMap::new();
                    for attr in attributes {
                        let attr_name = match attr.name.prefix {
                            Some(prefix) => format!("{}:{}", prefix, attr.name.local_name),
                            None => attr.name.local_name,
                        };
                        attr_map.insert(attr_name, attr.value);
                    }

                    let mut root = Element{
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        .. Element::default()
                    };
                    try!(root.parse(&mut reader));
                    doc.root = Some(root);

                },
                XmlEvent::EndDocument => break,
                _ => {},
            }
        }

        Ok(doc)

    }


}
