//! Context management for the VM execution.

use rpa_core::context::{Context, RetryConfig};
use rpa_core::value::Value;

/// Extension methods for the execution context.
pub trait ContextExt {
    /// Get a variable as a specific type, returning a default if not found.
    fn get_var_as_f64(&self, name: &str) -> Option<f64>;
    fn get_var_as_str(&self, name: &str) -> Option<&str>;
    fn get_var_as_bool(&self, name: &str) -> Option<bool>;
}

impl ContextExt for Context {
    fn get_var_as_f64(&self, name: &str) -> Option<f64> {
        self.get_var(name).and_then(|v| v.as_number())
    }

    fn get_var_as_str(&self, name: &str) -> Option<&str> {
        self.get_var(name).and_then(|v| v.as_str())
    }

    fn get_var_as_bool(&self, name: &str) -> Option<bool> {
        self.get_var(name).and_then(|v| v.as_bool())
    }
}