//! 窗口焦点控制 — WS_EX_NOACTIVATE + AttachThreadInput

use std::{thread, time::Duration};

use tauri::WebviewWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowLongW, GetWindowThreadProcessId, SetWindowLongW,
    ShowWindow, GWL_EXSTYLE, SW_HIDE, SW_SHOWNA, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

// FFI — GetFocus / SetFocus / AttachThreadInput (不在当前 windows feature 中)
extern "system" {
    fn GetFocus() -> isize;
    fn SetFocus(h: isize) -> isize;
    fn AttachThreadInput(id_attach: u32, id_attach_to: u32, f_attach: i32) -> i32;
    fn GetCurrentThreadId() -> u32;
}

/// 从 Tauri 的 HWND 提取原始 isize
///
/// Tauri 内部使用 windows v0.61，其 HWND 为 `*mut c_void`；
/// 我们的 v0.52 中 HWND 为 `isize`。通过指针转换桥接。
fn hwnd_isize(window: &WebviewWindow) -> Result<isize, String> {
    let hwnd = window.hwnd().map_err(|e| format!("获取窗口句柄: {}", e))?;
    // Tauri's HWND.0 是 *mut c_void → 转 isize
    let ptr: *mut std::ffi::c_void = unsafe { std::mem::transmute_copy(&hwnd) };
    Ok(ptr as isize)
}

/// 应用 WS_EX_NOACTIVATE + WS_EX_TOOLWINDOW
pub fn apply_no_activate_style(window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        let h: isize = hwnd_isize(window)?;
        let hwnd = HWND(h);
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let new_style = ex | WS_EX_NOACTIVATE.0 as i32 | WS_EX_TOOLWINDOW.0 as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        Ok(())
    }
}

/// ShowWindow(SW_SHOWNA) — 显示但不激活
pub fn show_no_activate(window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        ShowWindow(HWND(hwnd_isize(window)?), SW_SHOWNA);
        Ok(())
    }
}

/// 隐藏窗口
pub fn hide_window(window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        ShowWindow(HWND(hwnd_isize(window)?), SW_HIDE);
        Ok(())
    }
}

/// 完整流程：NOACTIVATE + SW_SHOWNA + AttachThreadInput
pub fn show_and_attach(window: &WebviewWindow) -> Result<(), String> {
    unsafe {
        let h: isize = hwnd_isize(window)?;

        // 1. 设置 NOACTIVATE 样式
        let hwnd = HWND(h);
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex | WS_EX_NOACTIVATE.0 as i32 | WS_EX_TOOLWINDOW.0 as i32);

        // 2. SW_SHOWNA 不激活
        ShowWindow(hwnd, SW_SHOWNA);

        // 3. 等窗口就绪
        thread::sleep(Duration::from_millis(30));

        // 4. AttachThreadInput + SetFocus → WebView 可接收键盘
        let foreground = GetForegroundWindow();
        if foreground.0 != 0 && h != 0 {
            let fg_tid = GetWindowThreadProcessId(foreground, None);
            let cb_tid = GetWindowThreadProcessId(hwnd, None);
            if fg_tid != cb_tid {
                AttachThreadInput(fg_tid, cb_tid, 1);
                SetFocus(h);
                AttachThreadInput(fg_tid, cb_tid, 0);
            }
        }

        Ok(())
    }
}

/// 恢复焦点到原窗口
pub fn restore_focus(foreground_hwnd: isize) {
    if foreground_hwnd == 0 {
        return;
    }
    unsafe {
        let self_tid = GetCurrentThreadId();
        let target_tid = GetWindowThreadProcessId(HWND(foreground_hwnd), None);
        if target_tid != self_tid {
            AttachThreadInput(self_tid, target_tid, 1);
        }
        SetFocus(foreground_hwnd);
        if target_tid != self_tid {
            AttachThreadInput(self_tid, target_tid, 0);
        }
    }
}
