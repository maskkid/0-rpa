//! Execution event types for monitoring and logging.

use serde::{Deserialize, Serialize};
use rpa_core::element::Rect;
use rpa_core::value::Value;

/// Events emitted during VM execution for monitoring, logging, and SSE streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// An instruction has started executing.
    InstructionStart {
        index: usize,
        description: String,
    },

    /// An instruction has completed successfully.
    InstructionComplete {
        index: usize,
        duration_ms: u64,
    },

    /// An instruction has failed.
    InstructionFailed {
        index: usize,
        error: String,
        will_retry: bool,
    },

    /// A context variable has been set.
    VariableSet {
        name: String,
        value: Value,
    },

    /// A workflow call has started.
    WorkflowCall {
        name: String,
    },

    /// A workflow call has returned.
    WorkflowReturn {
        name: String,
        result: Option<Value>,
    },

    /// An element find attempt using a specific strategy.
    FindAttempt {
        strategy: String,
    },

    /// An element find succeeded using a specific strategy.
    FindSuccess {
        strategy: String,
        duration_ms: u64,
    },

    /// A retry is being attempted.
    Retry {
        attempt: u32,
        max_retries: u32,
        delay_ms: u64,
        error: String,
    },

    // ──────────────────────────────
    // Debug Visualization Events
    // ──────────────────────────────

    /// A debug highlight was shown on screen.
    DebugHighlight {
        bounds: Rect,
        label: String,
        color: String,
        duration_ms: u64,
    },

    /// A debug screenshot was captured.
    DebugScreenshot {
        path: String,
        step_index: usize,
        before_or_after: String,
    },

    /// A debug action (click, mousemove, etc.) was performed.
    DebugAction {
        action_type: String,
        position: Option<(i32, i32)>,
        region: Option<Rect>,
        screenshot_path: Option<String>,
    },
}