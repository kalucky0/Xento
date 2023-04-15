use crate::value::{JsonNumber, JsonValue};
use alloc::string::String;
use core::fmt::{self, Write};

pub struct JsonSerializer<'a, W: Write + 'a> {
    writer: &'a mut W,
}

impl<'a, W: Write + 'a> JsonSerializer<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        JsonSerializer { writer }
    }

    pub fn serialize(&mut self, value: &JsonValue) -> fmt::Result {
        self.write_value(value)
    }

    fn write_value(&mut self, value: &JsonValue) -> fmt::Result {
        match value {
            JsonValue::Null => write!(self.writer, "null"),
            JsonValue::Boolean(true) => write!(self.writer, "true"),
            JsonValue::Boolean(false) => write!(self.writer, "false"),
            JsonValue::String(s) => self.write_string(s),
            JsonValue::Number(n) => self.write_number(n),
            JsonValue::Array(a) => self.write_array(a),
            JsonValue::Object(o) => self.write_object(o),
        }
    }

    fn write_string(&mut self, s: &str) -> fmt::Result {
        write!(self.writer, "\"")?;
        for c in s.chars() {
            match c {
                '\\' | '"' => write!(self.writer, "\\{}", c)?,
                '\n' => write!(self.writer, "\\n")?,
                '\r' => write!(self.writer, "\\r")?,
                '\t' => write!(self.writer, "\\t")?,
                _ => write!(self.writer, "{}", c)?,
            }
        }
        write!(self.writer, "\"")
    }

    fn write_number(&mut self, n: &JsonNumber) -> fmt::Result {
        match n {
            JsonNumber::Integer(i) => write!(self.writer, "{}", i),
            JsonNumber::Float(f) => write!(self.writer, "{}", f),
        }
    }

    fn write_array(&mut self, a: &[JsonValue]) -> fmt::Result {
        write!(self.writer, "[")?;
        let mut iter = a.iter();
        if let Some(first) = iter.next() {
            self.write_value(first)?;
            for value in iter {
                write!(self.writer, ", ")?;
                self.write_value(value)?;
            }
        }
        write!(self.writer, "]")
    }

    fn write_object(&mut self, o: &[(String, JsonValue)]) -> fmt::Result {
        write!(self.writer, "{{")?;
        let mut iter = o.iter();
        if let Some((key, value)) = iter.next() {
            self.write_string(key)?;
            write!(self.writer, ": ")?;
            self.write_value(value)?;
            for (key, value) in iter {
                write!(self.writer, ", ")?;
                self.write_string(key)?;
                write!(self.writer, ": ")?;
                self.write_value(value)?;
            }
        }
        write!(self.writer, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        serializer::JsonSerializer,
        value::{JsonNumber, JsonValue},
    };
    use alloc::{
        string::{String, ToString},
        vec,
    };

    #[test]
    fn test_serialize_null() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer.serialize(&JsonValue::Null).unwrap();
        assert_eq!(buffer, "null");
    }

    #[test]
    fn test_serialize_boolean() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer.serialize(&JsonValue::Boolean(true)).unwrap();
        assert_eq!(buffer, "true");

        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer.serialize(&JsonValue::Boolean(false)).unwrap();
        assert_eq!(buffer, "false");
    }

    #[test]
    fn test_serialize_string() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::String("Hello, world!".to_string()))
            .unwrap();
        assert_eq!(buffer, "\"Hello, world!\"");

        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::String(
                "A \"quoted\" string\nwith\ttabs and\r\nnewlines".to_string(),
            ))
            .unwrap();
        assert_eq!(
            buffer,
            "\"A \\\"quoted\\\" string\\nwith\\ttabs and\\r\\nnewlines\""
        );
    }

    #[test]
    fn test_serialize_number() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::Number(JsonNumber::Integer(42)))
            .unwrap();
        assert_eq!(buffer, "42");

        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::Number(JsonNumber::Float(3.14)))
            .unwrap();
        assert_eq!(buffer, "3.14");
    }

    #[test]
    fn test_serialize_array() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::Array(vec![
                JsonValue::Boolean(true),
                JsonValue::String("Hello, world!".to_string()),
                JsonValue::Number(JsonNumber::Integer(42)),
                JsonValue::Array(vec![]),
            ]))
            .unwrap();
        assert_eq!(buffer, "[true, \"Hello, world!\", 42, []]");
    }

    #[test]
    fn test_serialize_object() {
        let mut buffer = String::new();
        let mut serializer = JsonSerializer::new(&mut buffer);
        serializer
            .serialize(&JsonValue::Object(vec![
                ("foo".to_string(), JsonValue::Boolean(true)),
                (
                    "bar".to_string(),
                    JsonValue::String("Hello, world!".to_string()),
                ),
                (
                    "baz".to_string(),
                    JsonValue::Number(JsonNumber::Integer(42)),
                ),
                ("qux".to_string(), JsonValue::Array(vec![])),
            ]))
            .unwrap();
        assert_eq!(
            buffer,
            "{\"foo\": true, \"bar\": \"Hello, world!\", \"baz\": 42, \"qux\": []}"
        );
    }
}
