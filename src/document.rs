use std::fmt;
use std::io::{Read, Write};

use indexmap::IndexMap;

use crate::{Element, ElementBuilder, TreexmlError, XmlVersion};

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
            encoding: "UTF-8".to_string(),
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
    pub fn parse<R: Read>(r: R) -> Result<Document, TreexmlError> {
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

    pub fn write<W: Write>(&self, mut w: &mut W) -> Result<(), TreexmlError> {
        self.write_with(&mut w, true, "  ", true)
    }

    /// Writes a document to `w`
    pub fn write_with<W: Write>(
        &self,
        w: &mut W,
        document_decl: bool,
        indent_str: &'static str,
        indent: bool,
    ) -> Result<(), TreexmlError> {
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
