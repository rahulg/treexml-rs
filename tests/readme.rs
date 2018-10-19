extern crate treexml;

mod readme {

    use treexml::{Document, ElementBuilder as E};
    #[test]
    fn read() {
        let doc_raw = r#"
    <?xml version="1.1" encoding="UTF-8"?>
    <table>
        <fruit type="apple">worm</fruit>
        <vegetable />
    </table>
    "#;

        let doc = Document::parse(doc_raw.as_bytes()).unwrap();
        let root = doc.root.unwrap();

        let fruit = root.find_child(|tag| tag.name == "fruit").unwrap().clone();
        println!("{} [{:?}] = {:?}", fruit.name, fruit.attributes, fruit.text,);
    }

    #[test]
    fn write() {
        let mut something = E::new("something");
        something.attr("key", "value");
        something.text("some-text");

        let doc = Document::build(E::new("root").children(vec![E::new("list").children(vec![
            E::new("child").cdata("test data here"),
            E::new("child").attr("class", "foo").text("bar"),
            E::new("child").attr("class", 22).text(11),
            &mut E::new("child"),
            &mut something,
        ])]));

        println!("{}", doc);
    }
}
