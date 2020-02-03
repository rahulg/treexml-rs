use thiserror::Error;
use xml;

#[derive(Debug, Error)]
pub enum TreexmlError {
    #[error("Element not found: '{t}'")]
    ElementNotFound { t: String },
    #[error("Value could not be parsed: '{t}'")]
    ValueFromStr { t: String },
    #[error("Parse error: '{source}'")]
    ParseError {
        #[from]
        source: xml::reader::Error,
    },
    #[error("Write error: '{source}'")]
    WriteError {
        #[from]
        source: xml::writer::Error,
    },
}
