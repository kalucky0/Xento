use crate::value::{JsonNumber, JsonValue};
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

struct JsonParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser { input, pos: 0 }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) => {
                if c.is_digit(10) || c == '-' {
                    self.parse_number()
                } else {
                    Err(format!("Unexpected character: {}", c))
                }
            }
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        self.consume_str("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        if let Ok(_) = self.consume_char('t') {
            self.consume_str("rue")?;
            Ok(JsonValue::Boolean(true))
        } else {
            self.consume_str("alse")?;
            Ok(JsonValue::Boolean(false))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        let mut value = String::new();
        self.consume_char('"')?;
        loop {
            match self.next_char() {
                Some('\\') => value.push(self.parse_escaped_char()?),
                Some('"') => return Ok(JsonValue::String(value)),
                Some(c) => value.push(c),
                _ => return Err("Unexpected end of input".to_string()),
            }
        }
    }

    fn parse_escaped_char(&mut self) -> Result<char, String> {
        match self.next_char() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\u{0008}'),
            Some('f') => Ok('\u{000c}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.parse_unicode_escape(),
            Some(c) => Err(format!("Invalid escape character: {}", c)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let hex_chars = self.take_while(|c| c.is_ascii_hexdigit());
        let code_point = u32::from_str_radix(&hex_chars, 16)
            .map_err(|e| format!("Invalid Unicode escape: {}", e))?;
        char::from_u32(code_point)
            .ok_or_else(|| format!("Invalid Unicode code point: {}", code_point))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut value_str = String::new();
        let mut is_float = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() || c == '-' {
                value_str.push(c);
            } else if c == '.' || c == 'e' || c == 'E' {
                value_str.push(c);
                is_float = true;
            } else {
                break;
            }
            self.consume_char(c).unwrap();
        }
        if is_float {
            match value_str.parse::<f64>() {
                Ok(num) => Ok(JsonValue::Number(JsonNumber::Float(num))),
                Err(e) => Err(format!("Invalid float number: {}", e)),
            }
        } else {
            match value_str.parse::<i64>() {
                Ok(num) => Ok(JsonValue::Number(JsonNumber::Integer(num))),
                Err(e) => Err(format!("Invalid integer number: {}", e)),
            }
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        let mut arr = Vec::new();
        self.consume_char('[').unwrap();
        self.consume_whitespace();
        loop {
            if self.peek_char() == Some(']') {
                self.consume_char(']').unwrap();
                return Ok(JsonValue::Array(arr));
            }
            let value = self.parse_value()?;
            arr.push(value);
            self.consume_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.consume_char(',').unwrap();
                    self.consume_whitespace();
                }
                Some(']') => {}
                _ => {
                    return Err("Invalid array".to_string());
                }
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        let mut object = Vec::new();
        self.consume_char('{')?;
        self.consume_whitespace();

        while let Some(ch) = self.peek_char() {
            if ch == '}' {
                self.next_char();
                return Ok(JsonValue::Object(object));
            }

            if !object.is_empty() {
                self.consume_char(',')?;
                self.consume_whitespace();
            }

            let key = self.parse_string()?;
            self.consume_whitespace();
            self.consume_char(':')?;
            self.consume_whitespace();
            let value = self.parse_value()?;
            let key = match key {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            object.push((key, value));
            self.consume_whitespace();
        }

        Err("Unexpected end of input".to_string())
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.input.chars().nth(self.pos);
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn consume_str(&mut self, s: &str) -> Result<(), String> {
        for expected_ch in s.chars() {
            match self.next_char() {
                Some(ch) if ch == expected_ch => continue,
                Some(ch) => {
                    return Err(format!(
                        "1 Unexpected character: {}, expected: {}",
                        ch, expected_ch
                    ))
                }
                _ => return Err("Unexpected end of input".to_string()),
            }
        }
        Ok(())
    }

    fn consume_char(&mut self, expected_ch: char) -> Result<(), String> {
        match self.next_char() {
            Some(ch) if ch == expected_ch => Ok(()),
            Some(ch) => Err(format!(
                "Unexpected character: {}, expected: {}",
                ch, expected_ch
            )),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn take_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if predicate(ch) {
                s.push(ch);
                self.next_char();
            } else {
                break;
            }
        }
        s
    }
}

#[test]
fn test_parse_null() {
    let mut decoder = JsonParser::new("null");
    assert_eq!(decoder.parse_value().unwrap(), JsonValue::Null);
}

#[test]
fn test_parse_boolean_true() {
    let mut decoder = JsonParser::new("true");
    assert_eq!(decoder.parse_value().unwrap(), JsonValue::Boolean(true));
}

#[test]
fn test_parse_boolean_false() {
    let mut decoder = JsonParser::new("false");
    assert_eq!(decoder.parse_value().unwrap(), JsonValue::Boolean(false));
}

#[test]
fn test_parse_number_integer() {
    let mut decoder = JsonParser::new("42");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::Number(JsonNumber::Integer(42))
    );
}

#[test]
fn test_parse_number_negative_integer() {
    let mut decoder = JsonParser::new("-42");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::Number(JsonNumber::Integer(-42))
    );
}

#[test]
fn test_parse_number_float() {
    let mut decoder = JsonParser::new("3.14");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::Number(JsonNumber::Float(3.14))
    );
}

#[test]
fn test_parse_string() {
    let mut decoder = JsonParser::new("\"hello\"");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::String("hello".to_string())
    );
}

#[test]
fn test_parse_array() {
    let mut decoder = JsonParser::new("[1, 2, 3]");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::Array(vec![
            JsonValue::Number(JsonNumber::Integer(1)),
            JsonValue::Number(JsonNumber::Integer(2)),
            JsonValue::Number(JsonNumber::Integer(3)),
        ])
    );
}

#[test]
fn test_parse_object() {
    let mut decoder = JsonParser::new("{\"foo\": 1, \"bar\": true}");
    assert_eq!(
        decoder.parse_value().unwrap(),
        JsonValue::Object(vec![
            ("foo".to_string(), JsonValue::Number(JsonNumber::Integer(1))),
            ("bar".to_string(), JsonValue::Boolean(true)),
        ])
    );
}
