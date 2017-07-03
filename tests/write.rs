extern crate treexml;

mod write {

    mod document {

        use treexml::{Document, Element};

        #[test]
        fn simple_document() {

            let mut root = Element::new("root");
            let child = Element::new("child");
            root.children.push(child);

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let doc_ref = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<root>\n",
                "  <child />\n",
                "</root>"
            );

            assert_eq!(doc.to_string(), doc_ref);

        }

        #[test]
        fn condensed_document() {

            let mut root = Element::new("root");
            let child = Element::new("child");
            root.children.push(child);

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let mut condensed = vec![];
            doc.write_with(&mut condensed, false, "", false).unwrap();

            assert_eq!(
                String::from_utf8(condensed).unwrap(),
                "<root><child /></root>"
            );
        }

    }

    mod element {

        use treexml::{Document, Element};

        #[test]
        fn stringify() {

            let mut root = Element::new("root");
            let child = Element::new("child");
            let child2 = Element::new("child").clone();
            root.children.push(child);

            let _ = Document {
                root: Some(root),
                ..Document::default()
            };

            let elem_ref = "<child />";

            assert_eq!(child2.to_string(), elem_ref);

        }

    }

    mod contents {

        use treexml::{Document, Element};

        #[test]
        fn plain_text() {

            let mut root = Element::new("root");
            root.text = Some("text".to_owned());

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let doc_ref = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<root>text</root>",
            );

            assert_eq!(doc.to_string(), doc_ref);

        }

        #[test]
        fn tags_in_text() {

            let mut root = Element::new("root");
            root.text = Some("<tag />".to_owned());

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let doc_ref = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<root>&lt;tag /></root>",
            );

            assert_eq!(doc.to_string(), doc_ref);

        }

    }

    mod cdata {

        use treexml::{Document, Element};

        #[test]
        fn plain_text() {

            let mut root = Element::new("root");
            root.cdata = Some("data".to_owned());

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let doc_ref = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<root><![CDATA[data]]></root>",
            );

            assert_eq!(doc.to_string(), doc_ref);

        }

        #[test]
        fn nested_tags() {

            let mut root = Element::new("root");
            root.cdata = Some("<tag />".to_owned());

            let doc = Document {
                root: Some(root),
                ..Document::default()
            };

            let doc_ref = concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
                "<root><![CDATA[<tag />]]></root>",
            );

            assert_eq!(doc.to_string(), doc_ref);

        }

    }

}
