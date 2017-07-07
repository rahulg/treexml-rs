use super::*;

/// A builder for Element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementBuilder {
    /// The XML element we're working on
    element: Element,
}

impl ElementBuilder {
    /// Create a builder for an `Element` with the tag name `name`
    pub fn new<S>(name: S) -> ElementBuilder
    where
        S: ToString,
    {
        ElementBuilder {
            element: Element::new(name),
        }
    }

    /// Set the element's prefix to `prefix`
    pub fn prefix<S>(&mut self, prefix: S) -> &mut ElementBuilder
    where
        S: ToString,
    {
        self.element.prefix = Some(prefix.to_string());
        self
    }

    /// Set the element's attribute `key` to `value`
    pub fn attr<K, V>(&mut self, key: K, value: V) -> &mut ElementBuilder
    where
        K: ToString,
        V: ToString,
    {
        self.element
            .attributes
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set the element's text to `text`
    pub fn text<S>(&mut self, text: S) -> &mut ElementBuilder
    where
        S: ToString,
    {
        self.element.text = Some(text.to_string());
        self
    }

    /// Set the element's CDATA to `cdata`
    pub fn cdata<S>(&mut self, cdata: S) -> &mut ElementBuilder
    where
        S: ToString,
    {
        self.element.cdata = Some(cdata.to_string());
        self
    }

    /// Append children to this `Element`
    pub fn children(&mut self, children: Vec<&mut ElementBuilder>) -> &mut ElementBuilder {
        self.element
            .children
            .append(&mut children.into_iter().map(|i| i.element()).collect());
        self
    }

    /// Creates an `Element` from the builder
    pub fn element(&self) -> Element {
        self.element.clone()
    }
}

impl From<ElementBuilder> for Element {
    fn from(value: ElementBuilder) -> Element {
        value.element()
    }
}

impl From<Element> for ElementBuilder {
    fn from(element: Element) -> ElementBuilder {
        ElementBuilder { element }
    }
}
