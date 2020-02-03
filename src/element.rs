use std::borrow::Cow;
use std::fmt;
use std::io::{Read, Write};
use std::iter::Filter;
use std::slice::{Iter, IterMut};
use std::str::FromStr;
use std::string::ToString;

use indexmap::IndexMap;

use crate::{Document, TreexmlError};

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
    pub(crate) fn parse<R: Read>(
        &mut self,
        mut reader: &mut xml::reader::EventReader<R>,
    ) -> Result<(), TreexmlError> {
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
    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut xml::writer::EventWriter<W>,
    ) -> Result<(), TreexmlError> {
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
            name,
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
    pub fn find(&self, path: &str) -> Result<&Element, TreexmlError> {
        Self::find_path(&path.split('/').collect::<Vec<&str>>(), path, self)
    }

    pub fn find_value<T: FromStr>(&self, path: &str) -> Result<Option<T>, TreexmlError> {
        let el = self.find(path)?;
        if let Some(text) = el.text.as_ref() {
            match T::from_str(text) {
                Err(_) => Err(TreexmlError::ValueFromStr {
                    t: text.to_string(),
                }),
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
    ) -> Result<&'a Element, TreexmlError> {
        if path.is_empty() {
            return Ok(tree);
        }

        match tree.find_child(|t| t.name == path[0]) {
            Some(element) => Self::find_path(&path[1..], original, element),
            None => Err(TreexmlError::ElementNotFound { t: original.into() }),
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
