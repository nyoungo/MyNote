#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Code, ShortcutState};

mod commands;
mod domain;
mod infrastructure;
mod services;

#[cfg(target_os = "windows")]
mod mouse_hook;
#[cfg(target_os = "windows")]
mod keyboard_hook;
#[cfg(target_os = "windows")]
mod no_activate_window;
#[cfg(target_os = "windows")]
mod paste_executor;

use infrastructure::database::init_database;
use services::clipboard_monitor::ClipboardMonitor;
use services::clipboard_service::ClipboardService;

// ===== 应用状态 =====
#[cfg(target_os = "windows")]
pub struct AppState {
    mouse_hook: mouse_hook::MouseHookManager,
    keyboard_hook: keyboard_hook::KeyboardHookManager,
}
#[cfg(target_os = "windows")]
impl AppState {
    fn new() -> Self {
        Self {
            mouse_hook: mouse_hook::MouseHookManager::new(),
            keyboard_hook: keyboard_hook::KeyboardHookManager::new(),
        }
    }
}
#[cfg(not(target_os = "windows"))]
pub struct AppState;
#[cfg(not(target_os = "windows"))]
impl AppState {
    fn new() -> Self { Self }
}

// ===== 剪贴板面板命令 =====

#[cfg(target_os = "windows")]
#[tauri::command]
fn show_panel(app_handle: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let window = app_handle.get_webview_window("clipboard")
        .ok_or_else(|| "剪贴板窗口未找到".to_string())?;
    let _ = position_near_cursor(&window);
    no_activate_window::show_and_attach(&window)?;
    state.mouse_hook.start(&window, app_handle.clone())?;
    state.keyboard_hook.start(window.clone(), app_handle)?;
    let _ = window.emit("focus-search", ());
    println!("✅ 剪贴板面板已显示（不抢焦点）");
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn hide_panel(app_handle: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let window = app_handle.get_webview_window("clipboard")
        .ok_or_else(|| "剪贴板窗口未找到".to_string())?;
    state.mouse_hook.stop();
    state.keyboard_hook.stop();
    no_activate_window::hide_window(&window)?;
    println!("⏹️ 剪贴板面板已隐藏");
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn paste_item(
    app_handle: tauri::AppHandle,
    state: tauri::State<AppState>,
    content: String,
    content_type: String,
) -> Result<(), String> {
    state.mouse_hook.stop();
    state.keyboard_hook.stop();
    if let Some(window) = app_handle.get_webview_window("clipboard") {
        no_activate_window::hide_window(&window)?;
    }
    let _ = app_handle.emit("clipboard-hide", ());
    if content_type.starts_with("image/") {
        paste_executor::execute_paste_image(&content)?;
        println!("📋 已粘贴图片 ({} bytes base64)", content.len());
    } else {
        paste_executor::execute_paste(&content)?;
        println!("📋 已粘贴: {} 字符", content.len());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn position_near_cursor(window: &tauri::WebviewWindow) -> Result<(), String> {
    unsafe {
        let mut pt = std::mem::zeroed();
        if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt).is_ok() {
            // 获取光标所在的显示器
            let monitor = windows::Win32::Graphics::Gdi::MonitorFromPoint(
                pt,
                windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
            );
            let mut mi: windows::Win32::Graphics::Gdi::MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>() as u32;
            if windows::Win32::Graphics::Gdi::GetMonitorInfoW(monitor, &mut mi).as_bool() {
                let work = mi.rcWork;
                let win_w = 300.0;
                let win_h = 500.0;
                let mut x = pt.x as f64 - win_w / 2.0;
                let mut y = pt.y as f64 - win_h / 2.0;
                let left = work.left as f64;
                let top = work.top as f64;
                let right = work.right as f64;
                let bottom = work.bottom as f64;
                x = x.clamp(left, right - win_w);
                y = y.clamp(top, bottom - win_h);
                window
                    .set_position(tauri::LogicalPosition::new(x, y))
                    .map_err(|e| e.to_string())?;
            } else {
                // 后备：仅保护主屏幕边缘
                let sw = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                    windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
                );
                let sh = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                    windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
                );
                let x = ((pt.x - 150) as f64).clamp(0.0, (sw - 300) as f64);
                let y = ((pt.y - 150) as f64).clamp(0.0, (sh - 500) as f64);
                window
                    .set_position(tauri::LogicalPosition::new(x, y))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn show_panel() -> Result<(), String> { Ok(()) }
#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn hide_panel() -> Result<(), String> { Ok(()) }
#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn paste_item(_content: String, _content_type: String) -> Result<(), String> { Ok(()) }

// ===== 托盘图标 =====

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    // 托盘菜单项
    let show = MenuItemBuilder::with_id("show", "显示").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .items(&[&show, &quit])
        .build()?;

    // 托盘图标
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("MyNote")
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::TrayIconEvent;
            if let TrayIconEvent::DoubleClick { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

// ===== 应用入口 =====

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 拦截主窗口关闭 → 隐藏到托盘
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        // 全局快捷键：Ctrl+Shift+V
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app_handle, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app_handle.emit("toggle-clipboard", ());
                    }
                })
                .build()
        )
        .manage(ClipboardService::new())
        .manage(AppState::new())
        .setup(|app| {
            // 托盘图标
            setup_tray(app)?;

            // 注册全局快捷键
            let shortcut = tauri_plugin_global_shortcut::Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::KeyV,
            );
            app.global_shortcut().register(shortcut)
                .expect("注册全局快捷键 Ctrl+Shift+V 失败");

            // 初始化数据库
            let app_dir = app.path().app_data_dir().expect("获取应用数据目录失败");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("mynote.db");

            tauri::async_runtime::block_on(async {
                let pool = init_database(db_path.to_str().unwrap())
                    .await
                    .expect("初始化本地数据库失败");
                app.manage(pool.clone());
                println!("✅ 本地数据库已连接");

                let monitor = ClipboardMonitor::new();
                monitor.start(pool);
                app.manage(monitor);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::note::get_children,
            commands::note::create_node,
            commands::note::move_node,
            commands::note::load_note_content,
            commands::note::save_note_content,
            commands::note::update_node,
            commands::note::delete_node,
            commands::clipboard::get_cursor_position,
            commands::clipboard::get_clipboard_history,
            commands::clipboard::clear_clipboard_history,
            commands::clipboard::toggle_favorite,
            commands::clipboard::delete_clipboard_item,
            show_panel,
            hide_panel,
            paste_item,
        ])
        .run(tauri::generate_context!())
        .expect("启动 MyNote 失败");
}

fn main() {
    run();
}
