//! 粘贴执行器 — SendInput 模拟 Ctrl+V
//!
//! 写入系统剪贴板后模拟键盘 Ctrl+V，将内容粘贴到当前活动窗口。
//! 不改变窗口焦点，不依赖窗口激活状态。

use std::{thread, time::Duration};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_V,
};

/// 写入文本到剪贴板 + 模拟 Ctrl+V
pub fn execute_paste(content: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("初始化剪贴板: {}", e))?;
    clipboard
        .set_text(content)
        .map_err(|e| format!("写入文本失败: {}", e))?;

    thread::sleep(Duration::from_millis(50));
    unsafe { simulate_ctrl_v() }
}

/// 写入图片到剪贴板 + 模拟 Ctrl+V
///
/// `b64_png`: base64 编码的 PNG 数据
pub fn execute_paste_image(b64_png: &str) -> Result<(), String> {
    use base64::Engine;

    // 1. 解码 base64 → PNG 字节
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_png)
        .map_err(|e| format!("base64 解码失败: {}", e))?;

    // 2. 解码 PNG → RGBA 像素
    let img = image::load_from_memory(&png_bytes)
        .map_err(|e| format!("PNG 解码失败: {}", e))?
        .into_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.into_raw();

    // 3. 写入系统剪贴板
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("初始化剪贴板: {}", e))?;
    let img_data = arboard::ImageData {
        width: w as usize,
        height: h as usize,
        bytes: std::borrow::Cow::Owned(rgba),
    };
    clipboard
        .set_image(img_data)
        .map_err(|e| format!("写入图片失败: {}", e))?;

    thread::sleep(Duration::from_millis(50));
    unsafe { simulate_ctrl_v() }
}

/// SendInput 模拟 Ctrl+V 按键序列
unsafe fn simulate_ctrl_v() -> Result<(), String> {
    let ctrl_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(VK_CONTROL.0),
                wScan: 0x1D,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let v_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(VK_V.0),
                wScan: 0x2F,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let v_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(VK_V.0),
                wScan: 0x2F,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let ctrl_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(VK_CONTROL.0),
                wScan: 0x1D,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let inputs = [ctrl_down, v_down, v_up, ctrl_up];
    let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    if sent == 0 {
        return Err("SendInput 模拟 Ctrl+V 失败".to_string());
    }
    Ok(())
}
