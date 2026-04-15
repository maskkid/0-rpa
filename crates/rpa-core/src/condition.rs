//! Condition types for If and Loop instructions.

use serde::{Deserialize, Serialize};

use crate::target::Target;
use crate::value::Value;

/// A condition that can be evaluated in If and Loop instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Check if a target element exists on screen.
    ElementExists { target: Target },

    /// Check if a target element is visible on screen.
    ElementVisible { target: Target },

    /// Check if a context variable equals a value.
    VarEquals { var: String, value: Value },

    /// Check if a context variable is not empty/null.
    VarNotEmpty { var: String },

    /// Evaluate a JavaScript expression as a condition.
    Expression { js: String },

    /// All conditions must be true.
    And(Vec<Condition>),

    /// At least one condition must be true.
    Or(Vec<Condition>),

    /// Negate a condition.
    Not(Box<Condition>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_serializes() {
        let cond = Condition::ElementExists {
            target: Target::by_name("按钮"),
        };
        let json = serde_json::to_string(&cond).unwrap();
        let decoded: Condition = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Condition::ElementExists { .. }));
    }

    #[test]
    fn condition_and_or() {
        let cond = Condition::And(vec![
            Condition::VarEquals {
                var: "count".into(),
                value: Value::Number(5.0),
            },
            Condition::VarNotEmpty { var: "name".into() },
        ]);
        let json = serde_json::to_string(&cond).unwrap();
        let decoded: Condition = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, Condition::And(_)));
    }
}