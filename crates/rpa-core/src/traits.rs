//! Core traits that define the abstraction boundaries of the RPA system.

use async_trait::async_trait;

use crate::context::Context;
use crate::element::Element;
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