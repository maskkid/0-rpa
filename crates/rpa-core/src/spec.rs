//! Data extraction specifications.

use serde::{Deserialize, Serialize};

use crate::target::Target;

/// What attribute to extract from a UI element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtractAttribute {
    /// The text content of the element.
    Text,
    /// The value attribute of the element (for inputs).
    Value,
    /// The bounding rectangle of the element.
    Bounds,
    /// Whether a checkbox/toggle is checked.
    Checked,
    /// Whether a dropdown option is selected.
    Selected,
}

/// A single field to extract from a UI element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSpec {
    /// Variable name to store the extracted value.
    pub name: String,
    /// Target element to extract from.
    pub selector: Target,
    /// What attribute to extract.
    pub attribute: ExtractAttribute,
}

/// Specification for data extraction from UI elements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataSpec {
    pub fields: Vec<FieldSpec>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_spec_serializes() {
        let spec = DataSpec {
            fields: vec![FieldSpec {
                name: "message_text".into(),
                selector: Target::by_name("消息内容"),
                attribute: ExtractAttribute::Text,
            }],
        };
        let json = serde_json::to_string(&spec).unwrap();
        let decoded: DataSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, decoded);
    }
}