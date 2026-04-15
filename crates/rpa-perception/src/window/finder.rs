//! Windows window finder using Win32 APIs.
//!
//! Uses FindWindow, EnumWindows, and Process32First/Next to locate windows
//! by process name, window title, class name, or index.

use async_trait::async_trait;
use rpa_core::element::{Element, Rect};
use rpa_core::error::{RpaError, Result};
use rpa_core::target::WindowSelector;
use rpa_core::traits::WindowPerceptor;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT as WinRect};
use windows::Win32::System::Threading::{
    CreateToolhelp32Snapshot, OpenProcess, Process32FirstW, Process32NextW,
    PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, BringWindowToTop,
    FindWindowW, ENUMWINDOWSPROC,
};

/// Windows implementation of WindowPerceptor.
#[derive(Debug, Clone)]
pub struct WindowsWindowPerceptor;

impl WindowsWindowPerceptor {
    pub fn new() -> Self {
        Self
    }

    fn to_wide_string(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn from_wide_string(wide: &[u16]) -> String {
        let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
        String::from_utf16_lossy(&wide[..len])
    }

    /// Find process ID by process name.
    fn find_process_id_by_name(process_name: &str) -> Option<u32> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .ok()?;
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            let name_wide = Self::to_wide_string(process_name);

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let exe_name = Self::from_wide_string(&entry.szExeFile);
                    if exe_name.eq_ignore_ascii_case(&name_wide.iter().map(|&c| c as char).collect::<String>()) {
                        return Some(entry.th32ProcessID);
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            None
        }
    }

    /// Get window info from HWND.
    unsafe fn window_info(hwnd: HWND) -> Option<(String, String, Rect, u32)> {
        let mut title_len = 0;
        GetWindowTextLengthW(hwnd).map(|len| len as usize).ok()?;
        let mut title_buf = vec![0u16; title_len + 1];
        GetWindowTextW(hwnd, &mut title_buf).ok()?;

        let mut class_buf = vec![0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        if class_len == 0 {
            return None;
        }

        let mut rect = WinRect::default();
        GetWindowRect(hwnd, &mut rect).ok()?;

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        Some((
            Self::from_wide_string(&title_buf),
            Self::from_wide_string(&class_buf[..class_len as usize]),
            Rect::new(rect.left, rect.top, (rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32),
            process_id,
        ))
    }

    /// Find windows matching a selector via EnumWindows.
    fn find_windows_matching(selector: &WindowSelector) -> Vec<(HWND, String, String, Rect, u32)> {
        let process_id = selector.process_name.as_ref().and_then(|name| {
            Self::find_process_id_by_name(name)
        });

        let mut results = Vec::new();

        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let results = &mut *(lparam.0 as *mut Vec<(HWND, String, String, Rect, u32)>);

            if IsWindowVisible(hwnd).as_bool() {
                if let Some((title, class, rect, pid)) = WindowsWindowPerceptor::window_info(hwnd) {
                    results.push((hwnd, title, class, rect, pid));
                }
            }
            BOOL(1)
        }

        unsafe {
            let mut candidates: Vec<(HWND, String, String, Rect, u32)> = Vec::new();
            let _ = EnumWindows(
                Some(enum_callback),
                LPARAM(&mut candidates as *mut _ as isize),
            );

            // Filter by selector criteria
            for (hwnd, title, class, rect, pid) in candidates {
                if let Some(ref sel_class) = selector.class_name {
                    if !class.contains(sel_class) {
                        continue;
                    }
                }
                if let Some(ref sel_title) = selector.window_title {
                    if !title.contains(sel_title) {
                        continue;
                    }
                }
                if let Some(ref sel_process) = selector.process_name {
                    let exe_name = Self::get_process_name_by_pid(pid);
                    if !exe_name.to_lowercase().contains(&sel_process.to_lowercase()) {
                        continue;
                    }
                }
                if let Some(target_pid) = process_id {
                    if pid != target_pid {
                        continue;
                    }
                }
                results.push((hwnd, title, class, rect, pid));
            }
        }

        results
    }

    fn get_process_name_by_pid(pid: u32) -> String {
        unsafe {
            if let Ok(handle) = OpenProcess(windows::Win32::System::Threading::PROCESS_QUERY_INFORMATION | windows::Win32::System::Threading::PROCESS_VM_READ, false, pid) {
                let mut name_buf = vec![0u16; 260];
                let mut size = 260;
                if windows::Win32::System::Threading::QueryFullProcessImageNameW(handle, 0, windows::core::PWSTR(name_buf.as_mut_ptr()), &mut size).is_ok() {
                    let path = String::from_utf16_lossy(&name_buf[..size as usize]);
                    return std::path::Path::new(&path)
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                }
            }
        }
        String::new()
    }

    fn make_element(hwnd: HWND, title: String, class: String, rect: Rect, pid: u32) -> Element {
        let exe_name = Self::get_process_name_by_pid(pid);
        Element {
            id: format!("window_{:?}", hwnd.0),
            bounds: rect,
            text: Some(title.clone()),
            element_type: Some("Window".to_string()),
            platform_handle: Some(hwnd.0 as u64),
            process_id: Some(pid),
            process_name: Some(exe_name),
            window_title: Some(title),
        }
    }
}

impl Default for WindowsWindowPerceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WindowPerceptor for WindowsWindowPerceptor {
    async fn find_window(&self, selector: &WindowSelector) -> Result<Element> {
        let selector = selector.clone();

        tokio::task::spawn_blocking(move || {
            let matches = Self::find_windows_matching(&selector);

            if matches.is_empty() {
                return Err(RpaError::WindowNotFound(format!(
                    "No window found for selector: process={:?}, title={:?}, class={:?}",
                    selector.process_name, selector.window_title, selector.class_name
                )));
            }

            let idx = selector.index.unwrap_or(0) as usize;
            let match_info = matches.get(idx).ok_or_else(|| {
                RpaError::WindowNotFound(format!(
                    "Window index {} out of range (found {} windows)",
                    idx,
                    matches.len()
                ))
            })?;

            let (hwnd, title, class, rect, pid) = match_info;
            Ok(Self::make_element(*hwnd, title.clone(), class.clone(), *rect, *pid))
        })
        .await
        .map_err(|e| RpaError::Other(anyhow::anyhow!("Task join error: {}", e)))?
    }

    async fn find_all_windows(&self, selector: &WindowSelector) -> Result<Vec<Element>> {
        let selector = selector.clone();

        tokio::task::spawn_blocking(move || {
            let matches = Self::find_windows_matching(&selector);
            Ok(matches
                .into_iter()
                .map(|(hwnd, title, class, rect, pid)| {
                    Self::make_element(hwnd, title, class, rect, pid)
                })
                .collect())
        })
        .await
        .map_err(|e| RpaError::Other(anyhow::anyhow!("Task join error: {}", e)))?
    }

    async fn set_foreground(&self, element: &Element) -> Result<()> {
        let hwnd = windows::Win32::Foundation::HWND(
            element.platform_handle.unwrap() as *mut _
        );
        unsafe {
            SetForegroundWindow(hwnd)?;
            BringWindowToTop(hwnd)?;
        }
        Ok(())
    }

    async fn get_foreground_window(&self) -> Result<Element> {
        tokio::task::spawn_blocking(|| {
            unsafe {
                let hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
                if hwnd.0.is_null() {
                    return Err(RpaError::WindowNotFound("No foreground window".into()));
                }

                let (title, class, rect, pid) = Self::window_info(hwnd)
                    .ok_or_else(|| RpaError::WindowNotFound("Could not get foreground window info".into()))?;

                Ok(Self::make_element(hwnd, title, class, rect, pid))
            }
        })
        .await
        .map_err(|e| RpaError::Other(anyhow::anyhow!("Task join error: {}", e)))?
    }
}
