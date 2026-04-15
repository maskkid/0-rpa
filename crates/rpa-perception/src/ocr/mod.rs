//! OCR module for text recognition in screen regions.

#[cfg(target_os = "windows")]
pub mod windows_ocr;

#[cfg(target_os = "windows")]
pub use windows_ocr::WindowsOcrEngine;

/// Mock OCR engine for testing on non-Windows platforms.
pub mod mock {
    use async_trait::async_trait;
    use rpa_core::element::Rect;
    use rpa_core::error::{RpaError, Result};
    use rpa_core::traits::{OcrEngine, OcrResult};

    /// A mock OCR engine that returns configurable fake text.
    #[derive(Debug, Clone)]
    pub struct MockOcrEngine {
        pub fake_text: String,
        pub should_fail: bool,
    }

    impl MockOcrEngine {
        /// Create a new mock OCR engine with the given text.
        pub fn new(text: impl Into<String>) -> Self {
            Self {
                fake_text: text.into(),
                should_fail: false,
            }
        }

        /// Create a mock OCR engine that always fails.
        pub fn failing() -> Self {
            Self {
                fake_text: String::new(),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl OcrEngine for MockOcrEngine {
        async fn recognize(&self, _image_data: &[u8], region: Option<Rect>) -> Result<String> {
            if self.should_fail {
                return Err(RpaError::OcrFailed("Mock: OCR failed".into()));
            }

            let region_str = region
                .map(|r| format!(" (region: {:?}", r))
                .unwrap_or_default();
            Ok(format!("{}{}", self.fake_text, region_str))
        }

        async fn recognize_with_confidence(
            &self,
            _image_data: &[u8],
            region: Option<Rect>,
        ) -> Result<Vec<OcrResult>> {
            if self.should_fail {
                return Err(RpaError::OcrFailed("Mock: OCR failed".into()));
            }

            let region = region.unwrap_or(Rect::new(0, 0, 100, 50));
            Ok(vec![OcrResult {
                text: self.fake_text.clone(),
                confidence: 0.95,
                bounds: region,
            }])
        }
    }
}

pub use mock::MockOcrEngine;
