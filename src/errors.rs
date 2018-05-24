use failure;
use xml;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Element not found: '{}'", t)]
    ElementNotFound { t: String },
    #[fail(display = "Value could not be parsed: '{}'", t)]
    ValueFromStr { t: String },
    #[fail(display = "Parse error: '{}'", _0)]
    ParseError(failure::Error),
    #[fail(display = "Write error: '{}'", _0)]
    WriteError(failure::Error),
}

impl From<xml::reader::Error> for Error {
    fn from(v: xml::reader::Error) -> Self {
        Error::ParseError(v.into())
    }
}

impl From<xml::writer::Error> for Error {
    fn from(v: xml::writer::Error) -> Self {
        Error::WriteError(v.into())
    }
}
