//! Element and Rect types representing UI elements on screen.

use serde::{Deserialize, Serialize};

/// A rectangular area on screen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the center point of the rectangle.
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width as i32 / 2, self.y + self.height as i32 / 2)
    }

    /// Check if a point is inside this rectangle.
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.width as i32
            && py >= self.y
            && py < self.y + self.height as i32
    }
}

/// A UI element found on screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Unique identifier for this element.
    pub id: String,
    /// Bounding rectangle on screen.
    pub bounds: Rect,
    /// Visible text content of the element.
    pub text: Option<String>,
    /// UI element type (e.g., "Button", "Edit", "Text").
    pub element_type: Option<String>,
    /// Platform-specific handle (HWND on Windows).
    pub platform_handle: Option<u64>,
}

impl Element {
    /// Get the center point of this element's bounds.
    pub fn center(&self) -> (i32, i32) {
        self.bounds.center()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_center() {
        let rect = Rect::new(100, 200, 60, 40);
        assert_eq!(rect.center(), (130, 220));
    }

    #[test]
    fn rect_contains() {
        let rect = Rect::new(100, 200, 60, 40);
        assert!(rect.contains(130, 220));
        assert!(!rect.contains(50, 50));
    }
}