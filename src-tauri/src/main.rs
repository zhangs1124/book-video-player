#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};

const OVERLAY_JS: &str = include_str!("../overlay.js");
const START_URL: &str = "https://m.youtube.com";
const WIN_W: f64 = 400.0;
const WIN_H: f64 = 300.0;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let win = WebviewWindowBuilder::new(
                app,
                "main",
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
            let show = MenuItem::with_id(app, "show", "顯示", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "結束", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("YT Float")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
