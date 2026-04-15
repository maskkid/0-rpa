//! Mock actor for testing.

use rpa_core::element::Element;
use rpa_core::element::Rect;
use rpa_core::error::Result;
use rpa_core::instruction::{MouseButton, ScrollDirection};
use rpa_core::traits::Actor;

/// A mock actor that records actions for testing.
pub struct MockActor {
    actions: std::sync::Mutex<Vec<String>>,
}

impl MockActor {
    /// Create a new mock actor.
    pub fn new() -> Self {
        Self {
            actions: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get the recorded actions.
    pub fn actions(&self) -> Vec<String> {
        self.actions.lock().unwrap().clone()
    }
}

impl Default for MockActor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Actor for MockActor {
    async fn click(&self, element: &Element, button: MouseButton) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("click:{}:{:?}", element.id, button));
        Ok(())
    }

    async fn double_click(&self, element: &Element) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("double_click:{}", element.id));
        Ok(())
    }

    async fn input_text(&self, element: &Element, text: &str, clear_first: bool) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("input:{}:{}:clear={}", element.id, text, clear_first));
        Ok(())
    }

    async fn key_press(&self, key: &str, modifiers: Vec<rpa_core::instruction::ModifierKey>) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("key_press:{}:{:?}", key, modifiers));
        Ok(())
    }

    async fn scroll(&self, element: &Element, direction: ScrollDirection, amount: u32) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("scroll:{}:{:?}:{}", element.id, direction, amount));
        Ok(())
    }

    async fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("mousemove:{}:{}", x, y));
        Ok(())
    }

    async fn mouse_down(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("mousedown:{:?}:{}:{}", button, x, y));
        Ok(())
    }

    async fn mouse_up(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("mouseup:{:?}:{}:{}", button, x, y));
        Ok(())
    }

    async fn set_foreground(&self, element: &Element) -> Result<()> {
        self.actions
            .lock()
            .unwrap()
            .push(format!("setforeground:{}", element.id));
        Ok(())
    }

    async fn screenshot(&self, _region: Option<Rect>) -> Result<Vec<u8>> {
        self.actions
            .lock()
            .unwrap()
            .push("screenshot".into());
        Ok(vec![0x89, 0x50, 0x4E, 0x47]) // PNG magic bytes
    }
}