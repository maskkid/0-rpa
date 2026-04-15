//! RPA Action - Mouse and keyboard operations for UI automation.

pub mod mock;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::WindowsActor;