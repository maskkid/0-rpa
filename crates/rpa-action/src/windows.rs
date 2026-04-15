//! Windows actor implementation using Win32 SendInput and other APIs.

use async_trait::async_trait;
use rpa_core::element::{Element, Rect};
use rpa_core::error::{RpaError, Result};
use rpa_core::instruction::{MouseButton, ScrollDirection, ModifierKey};
use rpa_core::traits::Actor;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{POINT, HWND};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEINPUT, VK_BACK, VK_CONTROL, VK_MENU, VK_SHIFT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MOUSE_EVENT_FLAGS, MOUSE_EVENT_DATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SetCursorPos, SetForegroundWindow, BringWindowToTop,
    VkKeyScanW, MAPVK_VK_TO_VSC,
};

/// Windows implementation of the Actor trait.
#[derive(Debug, Clone)]
pub struct WindowsActor {
    screen_capturer: Option<Box<dyn rpa_core::traits::ScreenCapturer>>,
}

impl WindowsActor {
    /// Create a new WindowsActor.
    pub fn new() -> Self {
        Self {
            screen_capturer: None,
        }
    }

    /// Create a WindowsActor with a screen capturer.
    pub fn with_screen_capturer(capturer: impl rpa_core::traits::ScreenCapturer + 'static) -> Self {
        Self {
            screen_capturer: Some(Box::new(capturer)),
        }
    }

    /// Send a mouse event at the given screen coordinates.
    async fn send_mouse_event(
        &self,
        flags: MOUSE_EVENT_FLAGS,
        x: i32,
        y: i32,
    ) -> Result<()> {
        // Set cursor position
        unsafe {
            SetCursorPos(POINT { x, y })?;
        }

        // Small delay to ensure position is set
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Send the mouse event
        let inputs = [INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: x,
                    dy: y,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }];

        unsafe {
            let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if result != 1 {
                return Err(RpaError::Action(format!(
                    "SendInput failed to send {} events",
                    result
                )));
            }
        }

        Ok(())
    }

    /// Send a keyboard event.
    async fn send_key_event(&self, vk: u16, flags: KEYBD_EVENT_FLAGS) -> Result<()> {
        let inputs = [INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }];

        unsafe {
            let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if result != 1 {
                return Err(RpaError::Action(format!(
                    "SendInput failed to send keyboard event"
                )));
            }
        }

        Ok(())
    }
}

impl Default for WindowsActor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Actor for WindowsActor {
    async fn click(&self, element: &Element, button: MouseButton) -> Result<()> {
        let (x, y) = element.center();
        let flags = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP,
            MouseButton::Right => MOUSEEVENTF_RIGHTDOWN | MOUSEEVENTF_RIGHTUP,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN | MOUSEEVENTF_MIDDLEUP,
        };
        self.send_mouse_event(flags, x, y).await
    }

    async fn double_click(&self, element: &Element) -> Result<()> {
        let (x, y) = element.center();
        let flags = MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP;
        // Double click = two rapid clicks
        self.send_mouse_event(flags, x, y).await?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.send_mouse_event(flags, x, y).await
    }

    async fn input_text(&self, element: &Element, text: &str, clear_first: bool) -> Result<()> {
        // For now, just send the text as keystrokes
        // In a full implementation, this would use UI Automation to set the element text
        if clear_first {
            // Select all and delete
            self.key_press("a", vec![ModifierKey::Ctrl]).await?;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        for c in text.chars() {
            self.send_char_event(c).await?;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn key_press(&self, key: &str, modifiers: Vec<ModifierKey>) -> Result<()> {
        // Apply modifiers first
        for modifier in &modifiers {
            let vk = match modifier {
                ModifierKey::Ctrl => VK_CONTROL,
                ModifierKey::Alt => VK_MENU,
                ModifierKey::Shift => VK_SHIFT,
                ModifierKey::Super => 0x5B, // VK_LWIN
            };
            if vk != 0 {
                self.send_key_event(vk, KEYEVENTF_KEYUP).await?;
            }
        }

        // Look up the key
        let vk = self.virtual_key_code(key)?;
        self.send_key_event(vk, KEYEVENTF_KEYUP).await?;

        // Release modifiers in reverse
        for modifier in modifiers.iter().rev() {
            let vk = match modifier {
                ModifierKey::Ctrl => VK_CONTROL,
                ModifierKey::Alt => VK_MENU,
                ModifierKey::Shift => VK_SHIFT,
                ModifierKey::Super => 0x5B,
            };
            if vk != 0 {
                self.send_key_event(vk, KEYEVENTF_KEYUP).await?;
            }
        }

        Ok(())
    }

    async fn scroll(
        &self,
        element: &Element,
        direction: ScrollDirection,
        amount: u32,
    ) -> Result<()> {
        // Scroll using mouse wheel at element center
        let (x, y) = element.center();
        let (mouse_data, flags) = match direction {
            ScrollDirection::Up => (amount as i32 * -120, MOUSEEVENTF_WHEEL),
            ScrollDirection::Down => (amount as i32 * 120, MOUSEEVENTF_WHEEL),
            ScrollDirection::Left => (amount as i32 * -120, MOUSEEVENTF_HWHEEL),
            ScrollDirection::Right => (amount as i32 * 120, MOUSEEVENTF_HWHEEL),
        };

        unsafe {
            SetCursorPos(POINT { x, y })?;
        }

        let inputs = [INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: mouse_data as u32,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }];

        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        }

        Ok(())
    }

    async fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
        unsafe {
            SetCursorPos(POINT { x, y })?;
        }
        Ok(())
    }

    async fn mouse_down(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        let flags = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
            MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
        };
        self.send_mouse_event(flags, x, y).await
    }

    async fn mouse_up(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
        let flags = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTUP,
            MouseButton::Right => MOUSEEVENTF_RIGHTUP,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
        };
        self.send_mouse_event(flags, x, y).await
    }

    async fn set_foreground(&self, element: &Element) -> Result<()> {
        let hwnd = HWND(element.platform_handle.unwrap() as *mut _);
        unsafe {
            SetForegroundWindow(hwnd)?;
            BringWindowToTop(hwnd)?;
        }
        Ok(())
    }

    async fn screenshot(&self, region: Option<Rect>) -> Result<Vec<u8>> {
        match &self.screen_capturer {
            Some(capturer) => {
                if let Some(r) = region {
                    capturer.capture_region(r).await
                } else {
                    capturer.capture_screen().await
                }
            }
            None => Err(RpaError::Action(
                "No screen capturer registered".into(),
            )),
        }
    }
}

impl WindowsActor {
    /// Look up virtual key code from a key name.
    fn virtual_key_code(&self, key: &str) -> Result<u16> {
        // Handle special keys
        match key.to_uppercase().as_str() {
            "ENTER" | "RETURN" => Ok(0x0D),
            "TAB" => Ok(0x09),
            "ESCAPE" | "ESC" => Ok(0x1B),
            "BACKSPACE" => Ok(VK_BACK.0 as u16),
            "DELETE" => Ok(0x2E),
            "UP" => Ok(0x26),
            "DOWN" => Ok(0x28),
            "LEFT" => Ok(0x25),
            "RIGHT" => Ok(0x27),
            "HOME" => Ok(0x24),
            "END" => Ok(0x23),
            "PAGEUP" => Ok(0x21),
            "PAGEDOWN" => Ok(0x22),
            "F1" => Ok(0x70),
            "F2" => Ok(0x71),
            "F3" => Ok(0x72),
            "F4" => Ok(0x73),
            "F5" => Ok(0x74),
            "F6" => Ok(0x75),
            "F7" => Ok(0x76),
            "F8" => Ok(0x77),
            "F9" => Ok(0x78),
            "F10" => Ok(0x79),
            "F11" => Ok(0x7A),
            "F12" => Ok(0x7B),
            _ => {}
        }

        // Try to look up from VkKeyScan
        let char = key.chars().next().ok_or_else(|| {
            RpaError::Action(format!("Invalid key: {}", key))
        })?;

        let vk = unsafe { VkKeyScanW(char as u16) };
        if vk == 0xFFFF {
            return Err(RpaError::Action(format!("Unknown key: {}", key)));
        }

        Ok(vk as u16 & 0xFF)
    }

    /// Send a character event using SendInput.
    async fn send_char_event(&self, c: char) -> Result<()> {
        let vk = unsafe { VkKeyScanW(c as u16) };
        if vk == 0xFFFF {
            // Character notypeable, try direct scan code
            return self.send_key_event(c as u16, KEYEVENTF_KEYUP).await;
        }

        let vk_code = (vk & 0xFF) as u16;
        let shift = (vk >> 8) & 1 != 0;

        if shift {
            self.send_key_event(VK_SHIFT.0 as u16, KEYEVENTF_KEYUP).await?;
        }

        self.send_key_event(vk_code, KEYEVENTF_KEYUP).await?;

        if shift {
            self.send_key_event(VK_SHIFT.0 as u16, KEYEVENTF_KEYUP).await?;
        }

        Ok(())
    }
}
