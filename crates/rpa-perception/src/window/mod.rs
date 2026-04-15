//! Window perception module.
//!
//! Provides window finding and manipulation capabilities for applications
//! that don't expose standard UIA structure (games, Electron, Qt, DirectUI).

#[cfg(target_os = "windows")]
pub mod finder;

#[cfg(target_os = "windows")]
pub use finder::WindowsWindowPerceptor;

/// Mock window perceptor for testing on non-Windows platforms.
pub mod mock {
    use async_trait::async_trait;
    use rpa_core::element::Element;
    use rpa_core::error::{RpaError, Result};
    use rpa_core::target::WindowSelector;

    /// A mock window perceptor that returns configurable fake window elements.
    #[derive(Debug, Clone)]
    pub struct MockWindowPerceptor {
        pub windows: Vec<Element>,
        pub should_fail: bool,
    }

    impl MockWindowPerceptor {
        /// Create a new mock window perceptor with the given windows.
        pub fn new(windows: Vec<Element>) -> Self {
            Self {
                windows,
                should_fail: false,
            }
        }

        /// Create a mock perceptor that always fails to find windows.
        pub fn failing() -> Self {
            Self {
                windows: vec![],
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl rpa_core::traits::WindowPerceptor for MockWindowPerceptor {
        async fn find_window(&self, selector: &WindowSelector) -> Result<Element> {
            if self.should_fail {
                return Err(RpaError::WindowNotFound(format!(
                    "Mock: Window not found for selector: process={:?}, title={:?}, class={:?}",
                    selector.process_name, selector.window_title, selector.class_name
                )));
            }

            self.windows.first().cloned().ok_or_else(|| {
                RpaError::WindowNotFound("Mock: no windows registered".into())
            })
        }

        async fn find_all_windows(&self, selector: &WindowSelector) -> Result<Vec<Element>> {
            if self.should_fail {
                return Err(RpaError::WindowNotFound(format!(
                    "Mock: Windows not found for selector: process={:?}, title={:?}, class={:?}",
                    selector.process_name, selector.window_title, selector.class_name
                )));
            }
            Ok(self.windows.clone())
        }

        async fn set_foreground(&self, element: &Element) -> Result<()> {
            tracing::debug!("Mock set_foreground: {}", element.id);
            Ok(())
        }

        async fn get_foreground_window(&self) -> Result<Element> {
            self.windows.first().cloned().ok_or_else(|| {
                RpaError::WindowNotFound("Mock: no foreground window".into())
            })
        }
    }
}

pub use mock::MockWindowPerceptor;
