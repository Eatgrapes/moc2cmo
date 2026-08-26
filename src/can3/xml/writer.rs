use std::fmt::Write;

use quick_xml::escape::escape;

pub(crate) struct XmlWriter<'a> {
    output: &'a mut String,
}

impl<'a> XmlWriter<'a> {
    pub(crate) fn new(output: &'a mut String) -> Self {
        Self { output }
    }

    pub(crate) fn start(&mut self, tag: &str, attributes: &[(&str, String)]) {
        write!(self.output, "<{tag}").unwrap();
        self.attributes(attributes);
        self.output.push('>');
    }

    pub(crate) fn end(&mut self, tag: &str) {
        write!(self.output, "</{tag}>").unwrap();
    }

    pub(crate) fn empty(&mut self, tag: &str, attributes: &[(&str, String)]) {
        write!(self.output, "<{tag}").unwrap();
        self.attributes(attributes);
        self.output.push_str(" />");
    }

    pub(crate) fn text(&mut self, tag: &str, attributes: &[(&str, String)], value: impl ToString) {
        self.start(tag, attributes);
        self.output.push_str(&escape(value.to_string()));
        self.end(tag);
    }

    fn attributes(&mut self, attributes: &[(&str, String)]) {
        for (name, value) in attributes {
            write!(self.output, " {name}=\"{}\"", escape(value)).unwrap();
        }
    }
}

pub(crate) fn attr(name: &'static str, value: impl ToString) -> (&'static str, String) {
    (name, value.to_string())
}
