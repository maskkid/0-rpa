//! Dynamic value type for interop between workflows, JS, and external APIs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dynamic value type used throughout the system.
///
/// Supports null, bool, number, string, array, and object types,
/// similar to JSON values but usable in Rust with full type safety.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    /// Check if the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Try to get a bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get a number value.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get a string reference.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get an array reference.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Try to get an object reference.
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Look up a value in an object by key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object().and_then(|m| m.get(key))
    }
}

// Convenience From implementations

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_serialization_roundtrip() {
        let val = Value::Object(HashMap::from([
            ("name".into(), Value::String("test".into())),
            ("count".into(), Value::Number(42.0)),
            ("active".into(), Value::Bool(true)),
        ]));
        let json = serde_json::to_string(&val).unwrap();
        let decoded: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val, decoded);
    }

    #[test]
    fn value_convenience_conversions() {
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from(42), Value::Number(42.0));
        assert_eq!(Value::from("hello".to_string()), Value::String("hello".into()));
    }

    #[test]
    fn value_accessors() {
        let val = Value::Number(3.14);
        assert!(val.as_number().is_some());
        assert!(val.as_str().is_none());

        let val = Value::String("hello".into());
        assert_eq!(val.as_str(), Some("hello"));
    }
}