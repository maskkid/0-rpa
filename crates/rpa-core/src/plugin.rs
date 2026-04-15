//! Plugin manifest and permission types.

use serde::{Deserialize, Serialize};

/// Plugin manifest describing a plugin's metadata, capabilities, and permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entrypoints: PluginEntrypoints,
    #[serde(default)]
    pub permissions: Vec<Permission>,
}

/// Plugin entry points defining where workflows and scripts are located.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntrypoints {
    /// Path to the init script (relative to plugin root).
    pub init: Option<String>,
    /// Directory containing workflow JSON files.
    #[serde(default = "default_workflows_dir")]
    pub workflows_dir: String,
    /// Directory containing JS scripts.
    #[serde(default = "default_scripts_dir")]
    pub scripts_dir: String,
}

fn default_workflows_dir() -> String {
    "workflows/".into()
}

fn default_scripts_dir() -> String {
    "scripts/".into()
}

/// Permissions that a plugin can request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// Access to specific filesystem paths.
    FileSystem { paths: Vec<String> },
    /// Access to specific network hosts.
    Network { hosts: Vec<String> },
    /// Access to the system clipboard.
    Clipboard,
    /// Access to system information.
    SystemInfo,
    /// Custom permission with a string identifier.
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serializes() {
        let manifest = PluginManifest {
            name: "qianniu".into(),
            version: "1.0.0".into(),
            description: "Qianniu automation".into(),
            author: "test".into(),
            entrypoints: PluginEntrypoints {
                init: Some("scripts/init.js".into()),
                workflows_dir: "workflows/".into(),
                scripts_dir: "scripts/".into(),
            },
            permissions: vec![Permission::Clipboard],
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let decoded: PluginManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "qianniu");
        assert_eq!(decoded.permissions.len(), 1);
    }
}