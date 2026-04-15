//! Error types for the RPA system.

use thiserror::Error;

/// Core error type for the RPA system.
#[derive(Debug, Error)]
pub enum RpaError {
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Timeout after {0}ms waiting for {1}")]
    Timeout(u64, String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("JS runtime error: {0}")]
    JsRuntime(String),

    #[error("Perception failed: {0}")]
    Perception(String),

    #[error("Action failed: {0}")]
    Action(String),

    #[error("Context variable not found: {0}")]
    VarNotFound(String),

    #[error("Invalid instruction: {0}")]
    InvalidInstruction(String),

    #[error("Execution cancelled")]
    Cancelled,

    #[error("Workflow compilation error: {0}")]
    Compilation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Sandbox violation: {0}")]
    Sandbox(String),

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("OCR failed: {0}")]
    OcrFailed(String),

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias using RpaError.
pub type Result<T> = std::result::Result<T, RpaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = RpaError::ElementNotFound("按钮".into());
        assert_eq!(format!("{}", err), "Element not found: 按钮");

        let err = RpaError::Timeout(5000, "对话框".into());
        assert_eq!(format!("{}", err), "Timeout after 5000ms waiting for 对话框");
    }
}