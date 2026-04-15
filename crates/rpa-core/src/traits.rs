//! Core traits that define the abstraction boundaries of the RPA system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::element::{Element, Rect};
use crate::error::Result;
use crate::instruction::ModifierKey;
use crate::instruction::{MouseButton, ScrollDirection};
use crate::target::Target;
use crate::value::Value;

/// Trait for finding UI elements on screen.
///
/// Implementors provide platform-specific element finding capabilities
/// such as UI Automation, OCR, or image matching.
#[async_trait]
pub trait Perceptor: Send + Sync {
    /// Find a single element matching the target.
    async fn find(&self, target: &Target, ctx: &Context) -> Result<Element>;

    /// Find all elements matching the target.
    async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>>;
}

/// Trait for performing UI actions (click, type, etc).
///
/// Implementors provide platform-specific input simulation capabilities
/// such as SendInput on Windows or xdotool on Linux.
#[async_trait]
pub trait Actor: Send + Sync {
    /// Click on an element with the specified mouse button.
    async fn click(&self, element: &Element, button: MouseButton) -> Result<()>;

    /// Double-click on an element.
    async fn double_click(&self, element: &Element) -> Result<()>;

    /// Input text into an element.
    async fn input_text(&self, element: &Element, text: &str, clear_first: bool) -> Result<()>;

    /// Press a key with optional modifiers (e.g., Ctrl+C).
    async fn key_press(&self, key: &str, modifiers: Vec<ModifierKey>) -> Result<()>;

    /// Scroll on an element in a direction.
    async fn scroll(
        &self,
        element: &Element,
        direction: ScrollDirection,
        amount: u32,
    ) -> Result<()>;

    // ──────────────────────────────
    // Non-UIA / Mouse Operations
    // ──────────────────────────────

    /// Move the mouse cursor to absolute screen coordinates.
    async fn mouse_move(&self, x: i32, y: i32) -> Result<()>;

    /// Press a mouse button at absolute screen coordinates.
    async fn mouse_down(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;

    /// Release a mouse button at absolute screen coordinates.
    async fn mouse_up(&self, button: MouseButton, x: i32, y: i32) -> Result<()>;

    /// Bring a window to the foreground.
    async fn set_foreground(&self, element: &Element) -> Result<()>;

    /// Take a screenshot of the screen or a region.
    async fn screenshot(&self, region: Option<Rect>) -> Result<Vec<u8>>;
}

/// Trait for providing workflows (from plugins, filesystem, etc).
#[async_trait]
pub trait WorkflowProvider: Send + Sync {
    /// The name of this workflow provider.
    fn name(&self) -> &str;

    /// Get a workflow by name.
    fn get_workflow(&self, name: &str) -> Result<Vec<crate::instruction::Instruction>>;

    /// List available workflow names.
    fn list_workflows(&self) -> Vec<String>;
}

/// Trait for the JS runtime bridge.
#[async_trait]
pub trait JsRuntime: Send + Sync {
    /// Evaluate a JS script and return the result.
    async fn eval(&self, code: &str) -> Result<Value>;

    /// Call a JS function by name with arguments.
    async fn call_function(&self, name: &str, args: &[Value]) -> Result<Value>;

    /// Load a JS script file.
    async fn load_script(&self, path: &std::path::Path) -> Result<()>;

    /// Set a global variable in the JS context.
    async fn set_var(&self, name: &str, value: Value) -> Result<()>;

    /// Get a global variable from the JS context.
    async fn get_var(&self, name: &str) -> Result<Value>;
}

// ──────────────────────────────
// Window & Non-UIA Perception
// ──────────────────────────────

use crate::target::WindowSelector;

/// Trait for finding and interacting with windows.
///
/// Used for automating applications that don't expose standard UIA structure,
/// such as games, Electron apps, Qt apps, and DirectUI applications.
#[async_trait]
pub trait WindowPerceptor: Send + Sync {
    /// Find a single window matching the selector.
    async fn find_window(&self, selector: &WindowSelector) -> Result<Element>;

    /// Find all windows matching the selector.
    async fn find_all_windows(&self, selector: &WindowSelector) -> Result<Vec<Element>>;

    /// Bring a window to the foreground.
    async fn set_foreground(&self, element: &Element) -> Result<()>;

    /// Get the currently active (foreground) window.
    async fn get_foreground_window(&self) -> Result<Element>;
}

/// Result of an OCR recognition operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// The recognized text.
    pub text: String,
    /// Confidence score between 0.0 and 1.0.
    pub confidence: f64,
    /// Bounding rectangle of the recognized text in the image.
    pub bounds: Rect,
}

/// Trait for optical character recognition.
#[async_trait]
pub trait OcrEngine: Send + Sync {
    /// Recognize text in an image, optionally limited to a region.
    async fn recognize(&self, image_data: &[u8], region: Option<Rect>) -> Result<String>;

    /// Recognize text with per-result confidence and bounds.
    async fn recognize_with_confidence(
        &self,
        image_data: &[u8],
        region: Option<Rect>,
    ) -> Result<Vec<OcrResult>>;
}

/// Trait for capturing screenshots.
#[async_trait]
pub trait ScreenCapturer: Send + Sync {
    /// Capture the entire screen.
    async fn capture_screen(&self) -> Result<Vec<u8>>;

    /// Capture a specific region of the screen.
    async fn capture_region(&self, region: Rect) -> Result<Vec<u8>>;

    /// Capture a specific window.
    async fn capture_window(&self, element: &Element) -> Result<Vec<u8>>;
}

/// Highlight configuration for debug visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugHighlight {
    /// Bounding rectangle to highlight.
    pub bounds: Rect,
    /// Label to display near the highlight.
    pub label: String,
    /// Color of the highlight.
    pub color: DebugColor,
    /// How long to display the highlight in milliseconds.
    pub duration_ms: u64,
}

/// Color for debug highlights.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebugColor {
    Red,
    Green,
    Blue,
    Yellow,
}

/// Trait for debug visualization (overlays and annotated screenshots).
#[async_trait]
pub trait DebugCapturer: Send + Sync {
    /// Capture a screenshot with highlights drawn on it.
    ///
    /// Returns the path to the saved screenshot.
    async fn capture_with_highlight(
        &self,
        region: Option<Rect>,
        highlights: Vec<DebugHighlight>,
        save_path: &str,
    ) -> Result<String>;

    /// Show a highlight overlay on screen for a duration.
    async fn show_overlay(&self, highlights: Vec<DebugHighlight>) -> Result<()>;
}