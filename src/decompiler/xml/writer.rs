use std::fmt::Write;

use quick_xml::escape::escape;

pub(super) struct XmlWriter {
    output: String,
    depth: usize,
}

impl XmlWriter {
    pub(super) fn new() -> Self {
        Self {
            output: String::with_capacity(256 * 1024),
            depth: 0,
        }
    }

    pub(super) fn declaration(&mut self, value: &str) {
        self.output.push_str(value);
        self.output.push('\n');
    }

    pub(super) fn start(&mut self, tag: &str, attributes: &[(&str, String)]) {
        self.indent();
        self.output.push('<');
        self.output.push_str(tag);
        self.attributes(attributes);
        self.output.push_str(">\n");
        self.depth += 1;
    }

    pub(super) fn end(&mut self, tag: &str) {
        self.depth -= 1;
        self.indent();
        writeln!(self.output, "</{tag}>").expect("writing to String cannot fail");
    }

    pub(super) fn empty(&mut self, tag: &str, attributes: &[(&str, String)]) {
        self.indent();
        self.output.push('<');
        self.output.push_str(tag);
        self.attributes(attributes);
        self.output.push_str(" />\n");
    }

    pub(super) fn text(&mut self, tag: &str, attributes: &[(&str, String)], value: &str) {
        self.indent();
        self.output.push('<');
        self.output.push_str(tag);
        self.attributes(attributes);
        self.output.push('>');
        self.output.push_str(&escape(value));
        writeln!(self.output, "</{tag}>").expect("writing to String cannot fail");
    }

    pub(super) fn finish(self) -> Vec<u8> {
        self.output.into_bytes()
    }

    fn attributes(&mut self, attributes: &[(&str, String)]) {
        for (name, value) in attributes {
            self.output.push(' ');
            self.output.push_str(name);
            self.output.push_str("=\"");
            self.output.push_str(&escape(value));
            self.output.push('"');
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.output.push_str("  ");
        }
    }
}

pub(super) fn attr(name: &'static str, value: impl ToString) -> (&'static str, String) {
    (name, value.to_string())
}
