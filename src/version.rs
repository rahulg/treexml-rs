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
