//! 全局键盘钩子 — 检测任意按键，自动隐藏剪贴板面板

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, WebviewWindow};

struct KbTarget {
    window: WebviewWindow,
    app_handle: AppHandle,
}
static KB_TARGET: Mutex<Option<KbTarget>> = Mutex::new(None);
static KB_ACTIVE: AtomicBool = AtomicBool::new(false);
static KB_THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostThreadMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_KEYBOARD_LL,
};

const WM_KEYDOWN: u32 = 0x0100;
const WM_SYSKEYDOWN: u32 = 0x0104;

pub struct KeyboardHookManager;

impl KeyboardHookManager {
    pub fn new() -> Self { Self }

    /// 启动键盘钩子（独立线程 + 消息循环）
    pub fn start(&self, window: WebviewWindow, app_handle: AppHandle) -> Result<(), String> {
        if KB_ACTIVE.load(Ordering::SeqCst) {
            return Ok(());
        }

        *KB_TARGET.lock().unwrap() = Some(KbTarget { window, app_handle });
        KB_ACTIVE.store(true, Ordering::SeqCst);

        std::thread::spawn(|| unsafe {
            // 记录线程 ID，供 stop() 发消息唤醒
            extern "system" {
                fn GetCurrentThreadId() -> u32;
            }
            KB_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);

            // 短暂延迟，避免快捷键 Ctrl+Shift+V 中的 V 键触发立即隐藏
            std::thread::sleep(std::time::Duration::from_millis(200));

            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0);
            let hook = match hook {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("[kb_hook] SetWindowsHookExW FAILED: {}", e);
                    KB_ACTIVE.store(false, Ordering::SeqCst);
                    return;
                }
            };
            println!("[kb_hook] started");

            let mut msg = std::mem::zeroed();
            while KB_ACTIVE.load(Ordering::SeqCst) {
                let ret = windows::Win32::UI::WindowsAndMessaging::GetMessageW(
                    &mut msg, None, 0, 0,
                );
                if ret.0 == 0 {
                    break;
                }
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                let _ = windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
            KB_ACTIVE.store(false, Ordering::SeqCst);
            println!("[kb_hook] stopped");
        });

        Ok(())
    }

    /// 停止键盘钩子
    pub fn stop(&self) {
        KB_ACTIVE.store(false, Ordering::SeqCst);
        *KB_TARGET.lock().unwrap() = None;

        let tid = KB_THREAD_ID.swap(0, Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                let _ = PostThreadMessageW(tid, 0, WPARAM(0), LPARAM(0));
            }
        }
    }
}

/// 键盘钩子回调 — 仅 Escape 键隐藏面板
unsafe extern "system" fn keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let msg = w_param.0 as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
            // 从 KBDLLHOOKSTRUCT 读取 vkCode（第一个 DWORD 字段）
            let kb = l_param.0 as *const u32;
            let vk_code = *kb;

            // 仅 Escape 键触发隐藏（VK_ESCAPE = 0x1B）
            // Ctrl+Shift+V 等其他按键正常传递给窗口
            if vk_code == 0x1B {
                if let Ok(guard) = KB_TARGET.lock() {
                    if let Some(ref target) = *guard {
                        let _ = target.window.hide();
                        let _ = target.app_handle.emit("clipboard-hide", ());
                        println!("[kb_hook] Escape pressed, hiding panel");
                        KB_ACTIVE.store(false, Ordering::SeqCst);
                    }
                }
            }
        }
    }

    CallNextHookEx(None, code, w_param, l_param)
}
