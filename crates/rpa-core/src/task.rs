//! Task status and result types for the orchestrator.

use serde::{Deserialize, Serialize};

use crate::value::Value;

/// Status of an automation task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Task is waiting to be executed.
    Pending,
    /// Task is currently running.
    Running {
        /// Optional progress indicator (0.0 - 1.0).
        progress: Option<f64>,
    },
    /// Task completed successfully.
    Completed(TaskResult),
    /// Task failed with an error message.
    Failed(String),
    /// Task was cancelled by the user.
    Cancelled,
}

/// Result of a completed task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskResult {
    /// Optional output value from the workflow.
    pub output: Option<Value>,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
    /// Number of instructions executed.
    pub steps_executed: u32,
}

/// Priority levels for task scheduling.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_serializes() {
        let status = TaskStatus::Running { progress: Some(0.5) };
        let json = serde_json::to_string(&status).unwrap();
        let decoded: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, decoded);
    }

    #[test]
    fn task_result_serializes() {
        let result = TaskResult {
            output: Some(Value::String("done".into())),
            duration_ms: 1500,
            steps_executed: 10,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: TaskResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, decoded);
    }
}