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

/// Selector for finding windows by their properties.
///
/// Used to locate windows by process name, window title, class name, or index.
/// This enables automation of applications that don't expose standard UIA structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WindowSelector {
    /// Process name to match (e.g., "notepad.exe").
    pub process_name: Option<String>,
    /// Window title to match (substring match).
    pub window_title: Option<String>,
    /// Window class name to match.
    pub class_name: Option<String>,
    /// Zero-based index of the window to select if multiple matches exist.
    pub index: Option<u32>,
}

impl WindowSelector {
    /// Create a selector that matches by process name.
    pub fn by_process_name(name: impl Into<String>) -> Self {
        Self {
            process_name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Create a selector that matches by window title (substring match).
    pub fn by_title(title: impl Into<String>) -> Self {
        Self {
            window_title: Some(title.into()),
            ..Default::default()
        }
    }

    /// Create a selector that matches by window class name.
    pub fn by_class(class: impl Into<String>) -> Self {
        Self {
            class_name: Some(class.into()),
            ..Default::default()
        }
    }

    /// Set the zero-based index for selecting among multiple matches.
    pub fn index(mut self, idx: u32) -> Self {
        self.index = Some(idx);
        self
    }
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

    /// Find a window by its properties (process, title, class).
    Window { selector: WindowSelector },

    /// A region within a window, specified as an offset from the window's bounds.
    Region {
        window: Box<Target>,
        region: Rect,
    },
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

    /// Create a window target.
    pub fn window(selector: WindowSelector) -> Self {
        Target::Window { selector }
    }

    /// Create a region target (offset within a window).
    pub fn region(window: Target, region: Rect) -> Self {
        Target::Region {
            window: Box::new(window),
            region,
        }
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