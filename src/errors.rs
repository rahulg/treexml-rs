use xml;

error_chain!{
    errors {
        ElementNotFound(t: String) {
            description("Path returned no elements")
            display("Element not found: '{}'", t)
        }
        ValueFromStr(t: String) {
            description("Error parsing value")
            display("Value could not be parsed: '{}'", t)
        }
    }

    foreign_links {
        ParseError(xml::reader::Error);
        WriteError(xml::writer::Error);
    }
}
