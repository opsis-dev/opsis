use std::io::{self, Write};

use serde::Serialize;
use serde_json::ser::{CompactFormatter, Formatter, PrettyFormatter};

struct JsNumbers<F>(F);

impl<F: Formatter> Formatter for JsNumbers<F> {
    fn write_f32<W: ?Sized + Write>(&mut self, writer: &mut W, value: f32) -> io::Result<()> {
        self.write_f64(writer, f64::from(value))
    }

    fn write_f64<W: ?Sized + Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        writer.write_all(ryu_js::Buffer::new().format_finite(value).as_bytes())
    }

    fn begin_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.begin_array(writer)
    }

    fn end_array<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.end_array(writer)
    }

    fn begin_array_value<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        self.0.begin_array_value(writer, first)
    }

    fn end_array_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.end_array_value(writer)
    }

    fn begin_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.begin_object(writer)
    }

    fn end_object<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.end_object(writer)
    }

    fn begin_object_key<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        self.0.begin_object_key(writer, first)
    }

    fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.begin_object_value(writer)
    }

    fn end_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.0.end_object_value(writer)
    }
}

fn text_with<F: Formatter>(value: &impl Serialize, formatter: F) -> serde_json::Result<String> {
    let mut out = Vec::new();
    value.serialize(&mut serde_json::Serializer::with_formatter(
        &mut out,
        JsNumbers(formatter),
    ))?;
    String::from_utf8(out).map_err(|e| serde::ser::Error::custom(e.to_string()))
}

pub fn js_json(value: &impl Serialize) -> serde_json::Result<String> {
    text_with(value, CompactFormatter)
}

pub fn js_json_pretty(value: &impl Serialize) -> serde_json::Result<String> {
    text_with(value, PrettyFormatter::new())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn writes_numbers_as_json_stringify_does() {
        let value = json!({ "whole": 1920.0, "half": 0.5, "negative": -0.0, "big": 1e21, "small": 1e-7, "int": 3, "nan": f64::NAN, "list": [30.0, 2.25] });
        assert_eq!(
            js_json(&value).unwrap(),
            r#"{"whole":1920,"half":0.5,"negative":0,"big":1e+21,"small":1e-7,"int":3,"nan":null,"list":[30,2.25]}"#
        );
    }

    #[test]
    fn indents_by_two_spaces_like_json_stringify() {
        assert_eq!(
            js_json_pretty(&json!({ "a": [1.0, { "b": 2.5 }], "c": {} })).unwrap(),
            "{\n  \"a\": [\n    1,\n    {\n      \"b\": 2.5\n    }\n  ],\n  \"c\": {}\n}"
        );
    }
}
