//! Instruction types for the RPA VM.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::condition::Condition;
use crate::element::Rect;
use crate::spec::DataSpec;
use crate::target::Target;
use crate::value::Value;

/// Mouse button for click operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard modifier keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Super,
}

/// Scroll direction for scroll operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Log level for log instructions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// The core instruction enum representing all operations the VM can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    /// Click on a UI element.
    Click {
        target: Target,
        button: MouseButton,
    },

    /// Double-click on a UI element.
    DoubleClick {
        target: Target,
    },

    /// Input text into a UI element.
    Input {
        target: Target,
        text: String,
        clear_first: bool,
    },

    /// Press a key with optional modifiers.
    KeyPress {
        key: String,
        modifiers: Vec<ModifierKey>,
    },

    /// Extract data from a UI element into a context variable.
    Extract {
        target: Target,
        spec: DataSpec,
        into_var: String,
    },

    /// Wait for a fixed duration in milliseconds.
    Wait {
        duration_ms: u64,
    },

    /// Wait for a UI element to appear.
    WaitFor {
        target: Target,
        timeout_ms: u64,
        interval_ms: u64,
    },

    /// Call a workflow by name with arguments.
    Call {
        workflow: String,
        args: HashMap<String, Value>,
    },

    /// Loop over a body of instructions.
    Loop {
        max: Option<u32>,
        condition: Option<Condition>,
        body: Vec<Instruction>,
    },

    /// Conditional execution.
    If {
        condition: Condition,
        then_body: Vec<Instruction>,
        else_body: Option<Vec<Instruction>>,
    },

    /// Break out of the current loop.
    Break,

    /// Return a value from the current workflow.
    Return(Value),

    /// Log a message.
    Log {
        message: String,
        level: LogLevel,
    },

    /// Scroll on a UI element.
    Scroll {
        target: Target,
        direction: ScrollDirection,
        amount: u32,
    },

    // ──────────────────────────────
    // Non-UIA / Mouse Operations
    // ──────────────────────────────

    /// Move the mouse cursor to absolute screen coordinates.
    MouseMove {
        x: i32,
        y: i32,
    },

    /// Press a mouse button at absolute screen coordinates.
    MouseDown {
        button: MouseButton,
        x: i32,
        y: i32,
    },

    /// Release a mouse button at absolute screen coordinates.
    MouseUp {
        button: MouseButton,
        x: i32,
        y: i32,
    },

    /// Drag from one target position to another.
    Drag {
        from: Target,
        to: Target,
        button: MouseButton,
    },

    // ──────────────────────────────
    // Window Operations
    // ──────────────────────────────

    /// Bring a window to the foreground (activate it).
    SetForeground {
        target: Target,
    },

    /// Move and/or resize a window.
    MoveWindow {
        target: Target,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },

    // ──────────────────────────────
    // Screenshot & OCR
    // ──────────────────────────────

    /// Take a screenshot of the screen, a window, or a region.
    Screenshot {
        target: Option<Target>,
        region: Option<Rect>,
        save_path: Option<String>,
    },

    /// Perform OCR on a region and store the result in a variable.
    OcrRegion {
        target: Target,
        region: Rect,
        into_var: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_click_serializes() {
        let instr = Instruction::Click {
            target: Target::Position { x: 100, y: 200 },
            button: MouseButton::Left,
        };
        let json = serde_json::to_string(&instr).unwrap();
        let decoded: Instruction = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Instruction::Click { .. }));
    }

    #[test]
    fn instruction_loop_serializes() {
        let instr = Instruction::Loop {
            max: Some(10),
            condition: None,
            body: vec![Instruction::Wait { duration_ms: 500 }],
        };
        let json = serde_json::to_string(&instr).unwrap();
        let decoded: Instruction = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Instruction::Loop { .. }));
    }
}