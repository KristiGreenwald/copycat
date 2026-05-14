use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let show_hud_item = MenuItem::with_id(app, "show_hud", "Show HUD", true, None::<&str>)?;
    let clear_all_item = MenuItem::with_id(app, "clear_all", "Clear All Slots", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit ClipX", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hud_item, &settings_item, &clear_all_item, &quit_item])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("ClipX — Multi-Slot Clipboard")
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "settings" => {
                    if let Some(window) = app.get_webview_window("settings") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
                "show_hud" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("show-hud", ());
                    }
                }
                "clear_all" => {
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
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
