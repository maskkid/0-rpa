//! Mock perceptor for testing.

use rpa_core::context::Context;
use rpa_core::element::{Element, Rect};
use rpa_core::error::Result;
use rpa_core::target::Target;
use rpa_core::traits::Perceptor;

/// A mock perceptor that returns a configurable element.
pub struct MockPerceptor {
    default_element: Element,
    should_fail: bool,
}

impl MockPerceptor {
    /// Create a mock perceptor that returns the given element.
    pub fn new(id: &str) -> Self {
        Self {
            default_element: Element {
                id: id.to_string(),
                bounds: Rect::new(0, 0, 100, 50),
                text: Some(id.to_string()),
                element_type: Some("Button".to_string()),
                platform_handle: None,
                process_id: None,
                process_name: None,
                window_title: None,
            },
            should_fail: false,
        }
    }

    /// Create a mock perceptor that always fails.
    pub fn failing() -> Self {
        Self {
            default_element: Element {
                id: String::new(),
                bounds: Rect::new(0, 0, 0, 0),
                text: None,
                element_type: None,
                platform_handle: None,
                process_id: None,
                process_name: None,
                window_title: None,
            },
            should_fail: true,
        }
    }
}

#[async_trait::async_trait]
impl Perceptor for MockPerceptor {
    async fn find(&self, _target: &Target, _ctx: &Context) -> Result<Element> {
        if self.should_fail {
            Err(rpa_core::error::RpaError::ElementNotFound("mock fail".into()))
        } else {
            Ok(self.default_element.clone())
        }
    }

    async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>> {
        self.find(target, ctx).await.map(|el| vec![el])
    }
}