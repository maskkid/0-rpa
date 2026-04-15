//! Debug visualization module for overlays and annotated screenshots.

#[cfg(target_os = "windows")]
pub mod overlay;

#[cfg(target_os = "windows")]
pub use overlay::WindowsDebugCapturer;

/// Mock debug capturer for testing on non-Windows platforms.
pub mod mock {
    use async_trait::async_trait;
    use rpa_core::element::Rect;
    use rpa_core::error::{RpaError, Result};
    use rpa_core::traits::{DebugCapturer, DebugHighlight};

    /// A mock debug capturer that just logs calls.
    #[derive(Debug, Clone, Default)]
    pub struct MockDebugCapturer;

    impl MockDebugCapturer {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl DebugCapturer for MockDebugCapturer {
        async fn capture_with_highlight(
            &self,
            _region: Option<Rect>,
            highlights: Vec<DebugHighlight>,
            save_path: &str,
        ) -> Result<String> {
            tracing::debug!(
                "Mock capture_with_highlight: {} highlights, save_path={}",
                highlights.len(),
                save_path
            );
            Ok(save_path.to_string())
        }

        async fn show_overlay(&self, highlights: Vec<DebugHighlight>) -> Result<()> {
            tracing::debug!("Mock show_overlay: {} highlights", highlights.len());
            Ok(())
        }
    }
}

pub use mock::MockDebugCapturer;
