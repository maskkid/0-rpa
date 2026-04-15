//! RPA Perception - UI element finding via UIA, OCR, and image matching.

pub mod composite;
pub mod mock;

#[cfg(target_os = "windows")]
pub mod uia;

pub mod image;