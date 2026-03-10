use tauri::{AppHandle, SystemTrayEvent, Wry, menu::MenuItemIdRef};

pub async fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_ref() {
                "Quit" => {
                    app.exit(0);
                }
                "Connect" => {
                    // Show connection dialog or connect to default portal
                    if let Err(e) = show_connect_dialog(app).await {
                        log::error!("Failed to show connect dialog: {}", e);
                    }
                }
                "Disconnect" => {
                    if let Err(e) = disconnect_vpn(app).await {
                        log::error!("Failed to disconnect VPN: {}", e);
                    }
                }
                "Settings" => {
                    if let Err(e) = open_settings_window(app).await {
                        log::error!("Failed to open settings window: {}", e);
                    }
                }
                _ => {}
            }
        }
        SystemTrayEvent::LeftClick { .. } => {
            // Show main window on left click
            if let Err(e) = open_main_window(app).await {
                log::error!("Failed to open main window: {}", e);
            }
        }
        _ => {}
    }
}

async fn show_connect_dialog(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement connection dialog logic
    // For now, just open the main window
    open_main_window(app).await?;
    Ok(())
}

async fn disconnect_vpn(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let vpn_client = app.state::<std::sync::Arc<tokio::sync::Mutex<super::vpn_client::VpnClient>>>();
    let mut client = vpn_client.lock().await;
    client.disconnect().await?;
    Ok(())
}

async fn open_settings_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    open_main_window(app).await?;
    Ok(())
}

async fn open_main_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_window("main") {
        window.show()?;
        window.set_focus()?;
    } else {
        tauri::WindowBuilder::new(app, "main", tauri::WindowUrl::App("/".into()))
            .title("GlobalProtect VPN Client")
            .build()?;
    }
    Ok(())
}