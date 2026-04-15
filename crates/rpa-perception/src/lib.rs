//! RPA Perception - UI element finding via UIA, OCR, and image matching.

pub mod capture;
pub mod composite;
pub mod debug;
pub mod mock;
pub mod ocr;
pub mod window;

#[cfg(target_os = "windows")]
pub mod uia;

pub mod image;

#[cfg(target_os = "windows")]
pub use capture::WindowsScreenCapturer;

#[cfg(target_os = "windows")]
pub use debug::WindowsDebugCapturer;

#[cfg(target_os = "windows")]
pub use ocr::WindowsOcrEngine;

#[cfg(target_os = "windows")]
pub use window::WindowsWindowPerceptor;

pub use capture::MockScreenCapturer;
pub use debug::MockDebugCapturer;
pub use ocr::MockOcrEngine;
pub use window::MockWindowPerceptor;