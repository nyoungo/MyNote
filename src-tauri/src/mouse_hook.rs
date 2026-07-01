//! 全局鼠标钩子 — 检测窗口外点击，自动隐藏剪贴板面板

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ===== 全局状态（供 extern "system" 回调访问） =====
struct HookTarget {
    hwnd: isize,                    // 预先获取的 HWND 原始值
    app_handle: AppHandle,
}
static MOUSE_TARGET: Mutex<Option<HookTarget>> = Mutex::new(None);
static MOUSE_ACTIVE: AtomicBool = AtomicBool::new(false);
static MOUSE_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

// FFI — 获取窗口矩形（物理坐标，与 GetCursorPos 匹配）
extern "system" {
    fn GetWindowRect(hwnd: isize, rect: *mut RECT) -> i32;
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetCursorPos, PostThreadMessageW,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL,
};

const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_MBUTTONDOWN: u32 = 0x0207;
const WM_NCLBUTTONDOWN: u32 = 0x00A1;

pub struct MouseHookManager;

impl MouseHookManager {
    pub fn new() -> Self { Self }

    /// 启动鼠标钩子
    ///
    /// 预先获取窗口 HWND（isize），避免跨线程调用 Tauri APIs 失败。
    pub fn start(&self, window: &tauri::WebviewWindow, app_handle: AppHandle) -> Result<(), String> {
        if MOUSE_ACTIVE.load(Ordering::SeqCst) {
            return Ok(());
        }

        // 预先获取 HWND 原始值
        let hwnd: isize = window.hwnd()
            .map_err(|e| format!("获取窗口句柄失败: {}", e))
            .map(|h| unsafe { std::mem::transmute_copy(&h) })?;

        *MOUSE_TARGET.lock().unwrap() = Some(HookTarget { hwnd, app_handle });
        MOUSE_ACTIVE.store(true, Ordering::SeqCst);

        std::thread::spawn(move || unsafe {
            extern "system" { fn GetCurrentThreadId() -> u32; }
            MOUSE_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);

            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0);
            let hook = match hook {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[mouse_hook] SetWindowsHookExW FAILED: {}", e);
                    MOUSE_ACTIVE.store(false, Ordering::SeqCst);
                    return;
                }
            };
            println!("[mouse_hook] started (hwnd={:#x})", hwnd);

            let mut msg = std::mem::zeroed();
            while MOUSE_ACTIVE.load(Ordering::SeqCst) {
                let ret = windows::Win32::UI::WindowsAndMessaging::GetMessageW(
                    &mut msg, None, 0, 0,
                );
                if ret.0 == 0 { break; }
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                let _ = windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
            MOUSE_ACTIVE.store(false, Ordering::SeqCst);
            println!("[mouse_hook] stopped");
        });

        Ok(())
    }

    pub fn stop(&self) {
        MOUSE_ACTIVE.store(false, Ordering::SeqCst);
        *MOUSE_TARGET.lock().unwrap() = None;
        let tid = MOUSE_THREAD_ID.swap(0, Ordering::SeqCst);
        if tid != 0 {
            unsafe { let _ = PostThreadMessageW(tid, 0, WPARAM(0), LPARAM(0)); }
        }
    }
}

/// 鼠标钩子回调 — 检测窗口外点击
unsafe extern "system" fn mouse_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let msg = w_param.0 as u32;
        let is_click = matches!(
            msg,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_NCLBUTTONDOWN
        );

        if is_click {
            if let Ok(guard) = MOUSE_TARGET.lock() {
                if let Some(ref target) = *guard {
                    let mut pt = std::mem::zeroed();
                    let _ = GetCursorPos(&mut pt);

                    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    if GetWindowRect(target.hwnd, &mut rect) != 0 {
                        let outside = pt.x < rect.left || pt.x > rect.right
                            || pt.y < rect.top || pt.y > rect.bottom;

                        if outside {
                            let _ = target.app_handle.emit("clipboard-hide", ());
                            println!("[mouse_hook] outside click ({},{}), hiding", pt.x, pt.y);
                            MOUSE_ACTIVE.store(false, Ordering::SeqCst);
                        }
                    } else {
                        println!("[mouse_hook] GetWindowRect failed for hwnd={:#x}", target.hwnd);
                    }
                }
            }
        }
    }

    CallNextHookEx(None, code, w_param, l_param)
}
