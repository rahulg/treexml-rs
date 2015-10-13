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

mod element {

    use treexml::{Document, Element};

    #[test]
    fn find_child_none() {

        let doc_raw = r#"
        <root></root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let root = doc.root.unwrap();

        assert_eq!(root.find_child(|t| t.name == "child"), None);

    }

    #[test]
    fn find_child_one() {

        let doc_raw = r#"
        <root>
            <child attr_a="1" />
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let root = doc.root.unwrap();

        let mut child = Element::new("child");
        child.attributes.insert("attr_a".to_owned(), "1".to_owned());

        assert_eq!(root.find_child(|t| t.name == "child"), Some(&child));

    }

    #[test]
    fn find_child_many() {

        let doc_raw = r#"
        <root>
            <child attr_a="1" />
            <child attr_a="2" />
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let root = doc.root.unwrap();

        let mut child = Element::new("child");
        child.attributes.insert("attr_a".to_owned(), "1".to_owned());

        assert_eq!(root.find_child(|t| t.name == "child"), Some(&child));

    }

    #[test]
    fn find_child_mut_one() {

        let doc_raw = r#"
        <root>
            <child attr_a="1" />
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let mut root = doc.root.unwrap();

        {
            let mut child = root.find_child_mut(|t| t.name == "child").unwrap();
            let mut attr_a = child.attributes.get_mut(&"attr_a".to_owned()).unwrap();
            *attr_a = "2".to_owned();
        }

        let mut child = Element::new("child");
        child.attributes.insert("attr_a".to_owned(), "2".to_owned());

        assert_eq!(root.find_child(|t| t.name == "child"), Some(&child));

    }

    #[test]
    fn filter_children() {

        let doc_raw = r#"
        <root>
            <child>1</child>
            <child>2</child>
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let root = doc.root.unwrap();

        let mut ch1 = Element::new("child");
        let mut ch2 = Element::new("child");
        ch1.contents = Some("1".to_owned());
        ch2.contents = Some("2".to_owned());

        let children: Vec<&Element> = root.filter_children(|t| t.name == "child").collect();
        let children_ref = vec![&ch1, &ch2];

        assert_eq!(children, children_ref);

    }

    #[test]
    fn filter_children_mut() {

        let doc_raw = r#"
        <root>
            <child>1</child>
            <child>2</child>
        </root>
        "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let mut root = doc.root.unwrap();

        {
            let mut children: Vec<&mut Element> = root.filter_children_mut(|t| t.name == "child").collect();
            children[0].contents = Some("4".to_owned());
            children[1].contents = Some("5".to_owned());
        }

        let mut ch1 = Element::new("child");
        let mut ch2 = Element::new("child");
        ch1.contents = Some("4".to_owned());
        ch2.contents = Some("5".to_owned());

        let children: Vec<&Element> = root.filter_children(|t| t.name == "child").collect();
        let children_ref = vec![&ch1, &ch2];

        assert_eq!(children, children_ref);

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
