//! An element-tree style XML library

extern crate xml;

use std::collections::HashMap;
use std::io::Read;
use std::iter::Filter;
use std::slice::{Iter, IterMut};

use xml::reader::{EventReader, XmlEvent, Error};

/// An XML element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub prefix: Option<String>,
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Element>,
    pub contents: Option<String>,
}

/// An XML document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub version: XmlVersion,
    pub encoding: String,
    pub root: Option<Element>,
}

/// Enumeration of XML versions
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum XmlVersion {
    Version10,
    Version11,
}

impl Default for Element {
    fn default() -> Self {
        Element{
            prefix: None,
            name: "tag".to_owned(),
            attributes: HashMap::new(),
            children: Vec::new(),
            contents: None,
        }
    }
}

impl Element {

    /// Create a new element with the tag name `name`
    pub fn new<S>(name: S) -> Element where S: Into<String> {
        Element{name: name.into(), .. Element::default()}
    }

    /// Find a single child of the current element, given a predicate
    pub fn find_child<P>(&self, predicate: P) -> Option<&Element>
        where P: for<'r> Fn(&'r &Element) -> bool
    {
        self.children.iter().find(predicate)
    }

    /// Find a single child of the current element, given a predicate; returns a mutable borrow
    pub fn find_child_mut<P>(&mut self, predicate: P) -> Option<&mut Element>
        where P: for<'r> FnMut(&'r &mut Element) -> bool
    {
        self.children.iter_mut().find(predicate)
    }

    /// Filters the children of the current element, given a predicate
    pub fn filter_children<P>(&self, predicate: P) -> Filter<Iter<Element>, P>
        where P: for<'r> Fn(&'r &Element) -> bool
    {
        self.children.iter().filter(predicate)
    }

    /// Filters the children of the current element, given a predicate; returns a mutable iterator
    pub fn filter_children_mut<P>(&mut self, predicate: P) -> Filter<IterMut<Element>, P>
        where P: for<'r> FnMut(&'r &mut Element) -> bool
    {
        self.children.iter_mut().filter(predicate)
    }

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
    /// # Errors
    ///
    /// Passes any errors that the `xml-rs` library returns up the stack
    pub fn parse<R: Read>(r: R) -> Result<Document, Error> {

        let mut reader = EventReader::new(r);
        let mut doc = Document::new();

        loop {
            let ev = try!(reader.next());
            match ev {
                XmlEvent::StartDocument{version, encoding, ..} => {

                    // xml-rs's XmlVersion doesn't derive Debug *sadface*
                    doc.version = match version {
                        xml::common::XmlVersion::Version10 => XmlVersion::Version10,
                        xml::common::XmlVersion::Version11 => XmlVersion::Version11,
                    };
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

                    let root = Element{
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        children: Vec::new(),
                        contents: None,
                    };
                    doc.root = Some(try!(Document::parse_children(&mut reader, root)));

                },
                XmlEvent::EndDocument => break,
                _ => {},
            }
        }

        Ok(doc)

    }

    /// Internal recursive function to parse children of `element`
    fn parse_children<R: Read>(mut reader: &mut EventReader<R>, element: Element) -> Result<Element, Error> {

        let mut me = element.clone();

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

                    let child = Element{
                        prefix: name.prefix,
                        name: name.local_name,
                        attributes: attr_map,
                        children: Vec::new(),
                        contents: None
                    };
                    me.children.push(try!(Document::parse_children(&mut reader, child)));

                },
                XmlEvent::EndElement{name} => {

                    if name.prefix == me.prefix && name.local_name == me.name {
                        return Ok(me);
                    } else {
                        // This should never happen, since the base xml library will panic first
                        panic!("Unexpected closing tag: {}, expected {}", name, element.name);
                    }

                },
                XmlEvent::Characters(s) => {

                    let contents = match me.contents {
                        Some(v) => v,
                        None => String::new(),
                    };
                    me.contents = Some(contents + &s)

                },
                XmlEvent::CData(s) => {

                    let contents = match me.contents {
                        Some(v) => v,
                        None => String::new(),
                    };
                    me.contents = Some(contents + "<![CDATA[" + &s + "]]>");

                },
                XmlEvent::Whitespace(_) => {},
                XmlEvent::Comment(_) => {},
                _ => {},
            }
        }
    }

}
