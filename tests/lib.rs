extern crate treexml;

mod document {
    
    use treexml::{Document, XmlVersion};

    #[test]
    fn no_xml_tag() {

        let doc_raw = r#"
        <root>
            <child></child>
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();

        assert_eq!(doc.version, XmlVersion::Version10);
        assert_eq!(doc.encoding, "UTF-8".to_owned());
    }

    #[test]
    #[should_panic(expected = "Unexpected end of stream: no root element found")]
    fn empty() {
        let doc_raw = "";
        let _ = Document::parse(doc_raw.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic(expected = "Unexpected end of stream: no root element found")]
    fn no_root_tag() {

        let doc_raw = r#"
        <?xml version="1.1" encoding="UTF-8"?>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

}

mod tags {

    use treexml::Document;

    #[test]
    fn self_closing() {

        let doc_raw = r#"
        <root>
            <child />
        </root>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

    #[test]
    fn self_closing_no_space() {

        let doc_raw = r#"
        <root>
            <child/>
        </root>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

    #[test]
    #[should_panic(expected = "Unexpected closing tag: not_root, expected root")]
    fn mismatched_close() {

        let doc_raw = r#"
        <root></not_root>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

    #[test]
    #[should_panic(expected = "Unexpected closing tag: ROOT, expected root")]
    fn mismatched_case_close() {

        let doc_raw = r#"
        <root></ROOT>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

    #[test]
    #[should_panic(expected = "Unexpected end of stream: still inside the root element")]
    fn unclosed_root() {

        let doc_raw = r#"
        <root>
        "#;

        let _ = Document::parse(doc_raw.as_bytes()).unwrap();

    }

}

mod cdata {

    use treexml::Document;

    #[test]
    fn plain_text() {

        let doc_raw = r#"
        <root><![CDATA[data]]></root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();

        let root = doc.root.unwrap();

        assert_eq!(root.contents.unwrap(), "<![CDATA[data]]>".to_owned());

    }

    #[test]
    fn nested_tags() {

        let doc_raw = r#"
        <root><![CDATA[ <tag /> ]]></root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();

        let root = doc.root.unwrap();

        assert!(root.children.is_empty());
        assert_eq!(root.contents.unwrap(), "<![CDATA[ <tag /> ]]>".to_owned());

    }

}

mod complete {

    use treexml::{Document, Element, XmlVersion};

    #[test]
    fn parse_document() {

        let doc_raw = r#"
        <?xml version="1.1" encoding="UTF-8"?>
        <root>
            <child attr_a="1">content</child>
            <child attr_a="2"></child>
        </root>
        "#;

        let mut root = Element::new("root");

        let mut c1 = Element::new("child");
        c1.attributes.insert("attr_a".to_owned(), "1".to_owned());
        c1.contents = Some("content".to_owned());

        let mut c2 = Element::new("child");
        c2.attributes.insert("attr_a".to_owned(), "2".to_owned());

        root.children.push(c1);
        root.children.push(c2);

        let doc_ref = Document{
            version: XmlVersion::Version11,
            encoding: "UTF-8".to_owned(),
            root: Some(root),
        };

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();

        assert_eq!(doc, doc_ref);

    }

}
