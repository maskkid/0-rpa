//! Target types for locating UI elements.

use serde::{Deserialize, Serialize};

use crate::element::Rect;

/// Selector for UI Automation-based element finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiaSelector {
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub automation_id: Option<String>,
    pub control_type: Option<String>,
    pub index: Option<u32>,
}

/// Target specifies how to locate a UI element.
///
/// Targets are tried in order based on the configured find strategy:
/// UIA → OCR → Image → Position (fallback chain).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Target {
    /// Find element using Windows UI Automation.
    UIA { selector: UiaSelector },

    /// Find element by matching an image template.
    Image {
        path: String,
        threshold: Option<f64>,
    },

    /// Find element by matching text via OCR.
    Text {
        pattern: String,
        region: Option<Rect>,
    },

    /// Use a fixed screen position.
    Position { x: i32, y: i32 },
}

impl Target {
    /// Create a UIA target with just a name.
    pub fn by_name(name: impl Into<String>) -> Self {
        Target::UIA {
            selector: UiaSelector {
                name: Some(name.into()),
                class_name: None,
                automation_id: None,
                control_type: None,
                index: None,
            },
        }
    }

    /// Create a UIA target with an automation ID.
    pub fn by_automation_id(id: impl Into<String>) -> Self {
        Target::UIA {
            selector: UiaSelector {
                name: None,
                class_name: None,
                automation_id: Some(id.into()),
                control_type: None,
                index: None,
            },
        }
    }

    /// Create a position target.
    pub fn at(x: i32, y: i32) -> Self {
        Target::Position { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_uia_serializes() {
        let target = Target::by_name("发送按钮");
        let json = serde_json::to_string(&target).unwrap();
        let decoded: Target = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Target::UIA { .. }));
    }

    #[test]
    fn target_position_serializes() {
        let target = Target::at(100, 200);
        let json = serde_json::to_string(&target).unwrap();
        let decoded: Target = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Target::Position { x: 100, y: 200 }));
    }
}