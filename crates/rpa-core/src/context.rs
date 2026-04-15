//! Execution context: variables, call stack, and configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::value::Value;

/// Backoff strategy for retry delays.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackoffStrategy {
    /// Fixed delay between retries.
    Fixed,
    /// Linearly increasing delay: delay_ms * attempt.
    Linear,
    /// Exponentially increasing delay: base^attempt * delay_ms.
    Exponential { base: f64 },
}

/// Configuration for retry behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub delay_ms: u64,
    pub backoff: BackoffStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            delay_ms: 1000,
            backoff: BackoffStrategy::Exponential { base: 2.0 },
        }
    }
}

impl RetryConfig {
    /// Calculate the delay for a given retry attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        match &self.backoff {
            BackoffStrategy::Fixed => self.delay_ms,
            BackoffStrategy::Linear => self.delay_ms * (attempt as u64 + 1),
            BackoffStrategy::Exponential { base } => {
                (self.delay_ms as f64 * base.powi(attempt as i32)) as u64
            }
        }
    }
}

/// Execution context passed through the VM.
#[derive(Debug, Clone)]
pub struct Context {
    /// Context variables accessible by instructions.
    pub vars: HashMap<String, Value>,
    /// Workflow call stack for debugging.
    pub call_stack: Vec<String>,
    /// Retry configuration for this context.
    pub retry_config: RetryConfig,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            vars: HashMap::new(),
            call_stack: Vec::new(),
            retry_config: RetryConfig::default(),
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable in the context.
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        self.vars.insert(name.into(), value);
    }

    /// Get a variable from the context.
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    /// Push a workflow name onto the call stack.
    pub fn push_call(&mut self, workflow: String) {
        self.call_stack.push(workflow);
    }

    /// Pop a workflow name from the call stack.
    pub fn pop_call(&mut self) -> Option<String> {
        self.call_stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_config_delay() {
        let config = RetryConfig {
            max_retries: 3,
            delay_ms: 1000,
            backoff: BackoffStrategy::Exponential { base: 2.0 },
        };
        assert_eq!(config.delay_for_attempt(0), 1000);
        assert_eq!(config.delay_for_attempt(1), 2000);
        assert_eq!(config.delay_for_attempt(2), 4000);
    }

    #[test]
    fn context_vars() {
        let mut ctx = Context::new();
        ctx.set_var("count", Value::Number(5.0));
        assert_eq!(ctx.get_var("count"), Some(&Value::Number(5.0)));
        assert_eq!(ctx.get_var("missing"), None);
    }

    #[test]
    fn context_call_stack() {
        let mut ctx = Context::new();
        ctx.push_call("workflow_a".into());
        ctx.push_call("workflow_b".into());
        assert_eq!(ctx.pop_call(), Some("workflow_b".into()));
        assert_eq!(ctx.pop_call(), Some("workflow_a".into()));
    }
}