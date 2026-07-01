use tauri::State;
use serde::{Deserialize, Serialize};
use crate::services::clipboard_service::ClipboardService;
use crate::infrastructure::database::DbPool;

/// 将文本粘贴到当前活动窗口（Windows 平台通过 SendInput 模拟 Ctrl+V）
/// 特点：不切换窗口焦点，直接向系统发送键盘事件
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn paste_to_active_window(
    content: Option<String>,
    state: State<'_, ClipboardService>,
) -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        VK_CONTROL, VK_V, INPUT_0,
    };

    // 如果有内容，先写入系统剪贴板
    if let Some(text) = content {
        state.set_text(&text).await?;
    }

    unsafe {
        let down_ctrl = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let down_v = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let up_v = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_V,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let up_ctrl = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let inputs = [down_ctrl, down_v, up_v, up_ctrl];
        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
            return Err("SendInput 模拟按键失败".to_string());
        }
    }

    Ok(())
}

/// 非 Windows 平台的空实现
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn paste_to_active_window(
    _content: Option<String>,
    _state: State<'_, ClipboardService>,
) -> Result<(), String> {
    Err("该功能仅支持 Windows 平台".to_string())
}

/// 获取当前光标位置（用于定位剪贴板弹出窗口）
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_cursor_position() -> Result<(i32, i32), String> {
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows::Win32::Foundation::POINT;

    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point).is_ok() {
            Ok((point.x, point.y))
        } else {
            Err("获取光标位置失败".to_string())
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn get_cursor_position() -> Result<(i32, i32), String> {
    Err("该功能仅支持 Windows 平台".to_string())
}

/// 剪贴板历史记录模型（匹配数据库 clipboard_history 表）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: String,
    pub content: Option<String>,
    pub source_app: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub last_used_at: String,
}

/// 获取全部剪贴板历史记录（收藏优先，再按最后使用时间降序）
#[tauri::command]
pub async fn get_clipboard_history(
    db: State<'_, DbPool>,
    max_items: Option<i32>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = max_items.unwrap_or(200).max(10).min(500);
    sqlx::query_as::<_, ClipboardItem>(
        "SELECT * FROM clipboard_history \
         ORDER BY is_favorite DESC, last_used_at DESC LIMIT ?1"
    )
    .bind(limit)
    .fetch_all(&*db)
    .await
    .map_err(|e| e.to_string())
}

/// 清空全部剪贴板历史记录
#[tauri::command]
pub async fn clear_clipboard_history(
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM clipboard_history")
        .execute(&*db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// 切换收藏状态（置顶/取消置顶）
#[tauri::command]
pub async fn toggle_favorite(
    id: String,
    db: State<'_, DbPool>,
) -> Result<bool, String> {
    // 先获取当前状态
    let current: Option<bool> = sqlx::query_scalar(
        "SELECT is_favorite FROM clipboard_history WHERE id = ?1"
    )
    .bind(&id)
    .fetch_optional(&*db)
    .await
    .map_err(|e| e.to_string())?;

    match current {
        Some(val) => {
            let new_val = !val;
            sqlx::query("UPDATE clipboard_history SET is_favorite = ?1 WHERE id = ?2")
                .bind(new_val)
                .bind(&id)
                .execute(&*db)
                .await
                .map_err(|e| e.to_string())?;
            println!("[toggle_favorite] id={} -> {}", &id[..8], new_val);
            Ok(new_val)
        }
        None => Err("条目不存在".to_string()),
    }
}

/// 删除单个剪贴板历史条目
#[tauri::command]
pub async fn delete_clipboard_item(
    id: String,
    db: State<'_, DbPool>,
) -> Result<(), String> {
    sqlx::query("DELETE FROM clipboard_history WHERE id = ?1")
        .bind(&id)
        .execute(&*db)
        .await
        .map_err(|e| e.to_string())?;
    println!("[delete_clipboard_item] id={}", &id[..8]);
    Ok(())
}
