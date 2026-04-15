//! Screen capture module for screenshots.

#[cfg(target_os = "windows")]
pub mod screen;

#[cfg(target_os = "windows")]
pub use screen::WindowsScreenCapturer;

/// Mock screen capturer for testing on non-Windows platforms.
pub mod mock {
    use async_trait::async_trait;
    use rpa_core::element::{Element, Rect};
    use rpa_core::error::{RpaError, Result};
    use rpa_core::traits::ScreenCapturer;

    /// A mock screen capturer that returns a 1x1 PNG.
    #[derive(Debug, Clone, Default)]
    pub struct MockScreenCapturer;

    impl MockScreenCapturer {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ScreenCapturer for MockScreenCapturer {
        async fn capture_screen(&self) -> Result<Vec<u8>> {
            // Return a minimal valid PNG (1x1 transparent pixel)
            Ok(vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
                0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // 8-bit RGBA
                0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
                0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, // compressed data
                0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, // end marker
                0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND chunk
                0x42, 0x60, 0x82,                                 // CRC
            ])
        }

        async fn capture_region(&self, _region: Rect) -> Result<Vec<u8>> {
            self.capture_screen().await
        }

        async fn capture_window(&self, _element: &Element) -> Result<Vec<u8>> {
            self.capture_screen().await
        }
    }
}

pub use mock::MockScreenCapturer;
