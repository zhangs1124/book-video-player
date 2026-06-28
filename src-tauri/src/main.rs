#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const OVERLAY_JS: &str = include_str!("../overlay.js");
const START_URL: &str = "https://m.youtube.com";
const WIN_W: f64 = 400.0;
const WIN_H: f64 = 300.0;

// 全域 window 計數器，用來生成唯一的 window ID
static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(0);

// 老闆鍵：切換所有視窗顯示/隱藏；隱藏時暫停影片(避免聲音穿幫)
fn toggle_boss<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let windows = app.webview_windows();
    
    if windows.is_empty() {
        return;
    }

    // 檢查是否有任何視窗是可見的
    let any_visible = windows.iter().any(|(_, win)| win.is_visible().unwrap_or(true));

    // 如果有任何視窗可見，隱藏所有；否則顯示所有
    for (_, win) in windows.iter() {
        if any_visible {
            let _ = win.eval(
                "document.querySelectorAll('video').forEach(function(v){try{v.pause()}catch(e){}})",
            );
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        toggle_boss(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 生成唯一的 window ID
            let window_id = format!(
                "main_{}",
                WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst)
            );

            let win = WebviewWindowBuilder::new(
                app,
                &window_id,
                WebviewUrl::External(START_URL.parse().unwrap()),
            )
            .title("YT Float")
            .decorations(false)
            .always_on_top(true)
            .resizable(true)
            .inner_size(WIN_W, WIN_H)
            .min_inner_size(220.0, 160.0)
            .initialization_script(OVERLAY_JS)
            .build()?;

            // 預設放到螢幕右下角
            if let Ok(Some(monitor)) = win.current_monitor() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let pw = WIN_W * scale;
                let ph = WIN_H * scale;
                let x = size.width as f64 - pw - 24.0;
                let y = size.height as f64 - ph - 64.0;
                let _ = win.set_position(PhysicalPosition::new(x.max(0.0), y.max(0.0)));
            }

            // 系統匣
            let show = MenuItem::with_id(app, "show", "顯示全部", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "隱藏全部", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "結束全部", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &hide, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("YT Float")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        for (_, w) in app.webview_windows().iter() {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        for (_, w) in app.webview_windows().iter() {
                            let _ = w.eval(
                                "document.querySelectorAll('video').forEach(function(v){try{v.pause()}catch(e){}})",
                            );
                            let _ = w.hide();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // 註冊老闆鍵 Ctrl+Shift+Z（全域）
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
                let boss = Shortcut::new(
                    Some(Modifiers::CONTROL | Modifiers::SHIFT),
                    Code::KeyZ,
                );
                app.global_shortcut().register(boss)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
