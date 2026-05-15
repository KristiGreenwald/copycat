use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::WebviewWindowBuilder,
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let show_hud_item = MenuItem::with_id(app, "show_hud", "Show HUD", true, None::<&str>)?;
    let clear_all_item = MenuItem::with_id(app, "clear_all", "Clear All Slots", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit CopyCat", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hud_item, &settings_item, &clear_all_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("CopyCat — Multi-Slot Clipboard")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "settings" => {
                    eprintln!("[CopyCat] Tray: Settings clicked");
                    if let Some(window) = app.get_webview_window("settings") {
                        window.show().ok();
                        window.set_focus().ok();
                    } else {
                        let html = include_str!("../../html/settings.html");
                        let data_url = format!(
                            "data:text/html;charset=utf-8,{}",
                            urlencoding::encode(html)
                        );
                        match WebviewWindowBuilder::new(
                            app,
                            "settings",
                            tauri::WebviewUrl::External(data_url.parse().unwrap()),
                        )
                        .title("CopyCat Settings")
                        .inner_size(700.0, 600.0)
                        .resizable(true)
                        .build()
                        {
                            Ok(_) => eprintln!("[CopyCat] Settings window created"),
                            Err(e) => eprintln!("[CopyCat] Failed to create settings: {}", e),
                        }
                    }
                }
                "show_hud" => {
                    eprintln!("[CopyCat] Tray: Show HUD clicked");
                    let occupied = if let Some(state) = app.try_state::<crate::clipboard::manager::SharedClipboardManager>() {
                        state.lock().unwrap().get_occupied_slots().len()
                    } else { 0 };
                    crate::hotkeys::manager::show_hud_window(app, occupied);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("show-hud", ());
                    }
                }
                "clear_all" => {
                    eprintln!("[CopyCat] Tray: Clear All clicked");
                    if let Some(state) = app.try_state::<crate::clipboard::manager::SharedClipboardManager>() {
                        let mut mgr = state.lock().unwrap();
                        mgr.clear_all();
                        crate::clipboard::persistence::save_slots(mgr.slots_for_persistence()).ok();
                    }
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("slots-updated", ());
                    }
                }
                "quit" => {
                    eprintln!("[CopyCat] Tray: Quit clicked");
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
