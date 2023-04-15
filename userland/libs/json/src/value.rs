use alloc::{
    string::String,
    vec::Vec,
};

#[derive(Debug, PartialEq)]
pub enum JsonNumber {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}