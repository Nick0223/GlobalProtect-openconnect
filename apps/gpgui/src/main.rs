// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tray;
mod portal_manager;
mod auth_manager;
mod vpn_client;
mod config;
mod auto_connect;

use tauri::{
    AppHandle, Manager, SystemTray, SystemTrayEvent, WindowEvent, Wry,
    menu::{Menu, MenuItem, Submenu},
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Initialize configuration
    let config = Arc::new(Mutex::new(config::Config::load().await?));
    
    // Initialize portal manager
    let portal_manager = Arc::new(Mutex::new(portal_manager::PortalManager::new()));
    
    // Initialize VPN client
    let vpn_client = Arc::new(Mutex::new(vpn_client::VpnClient::new()));
    
    // Build system tray menu
    let tray_menu = build_tray_menu()?;
    
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit_all("single-instance", Payload { message: "App is already running!".to_string() }).unwrap();
        }))
        .setup(|app| {
            // Store shared state in app handle
            app.manage(config.clone());
            app.manage(portal_manager.clone());
            app.manage(vpn_client.clone());
            
            // Initialize auto-connect if enabled
            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                if let Ok(config) = config.lock().await.load().await {
                    if config.auto_connect && !config.portals.is_empty() {
                        let first_portal = config.portals.first().unwrap().clone();
                        // TODO: Implement auto-connect logic
                    }
                }
            });
            
            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| {
            tauri::async_runtime::block_on(tray::handle_tray_event(app, event));
        })
        .invoke_handler(tauri::generate_handler![
            connect_vpn,
            disconnect_vpn,
            get_portals,
            add_portal,
            remove_portal,
            update_portal,
            get_current_status,
            enable_auto_connect,
            disable_auto_connect,
            open_settings,
            quit_app
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });

    Ok(())
}

fn build_tray_menu() -> Result<Menu, Box<dyn std::error::Error>> {
    let quit = MenuItem::new("Quit", true, None::<&str>)?;
    let connect = MenuItem::new("Connect", true, None::<&str>)?;
    let disconnect = MenuItem::new("Disconnect", true, None::<&str>)?;
    let settings = MenuItem::new("Settings", true, None::<&str>)?;
    
    let connection_menu = Submenu::new("Connection", Menu::new().add_item(connect).add_item(disconnect))?;
    let preferences_menu = Submenu::new("Preferences", Menu::new().add_item(settings))?;
    
    let menu = Menu::new()
        .add_submenu(connection_menu)
        .add_submenu(preferences_menu)
        .add_item(quit);
    
    Ok(menu)
}

// Tauri commands
#[tauri::command]
async fn connect_vpn(
    app_handle: AppHandle,
    portal_url: String,
    gateway_address: Option<String>,
) -> Result<String, String> {
    // Get VPN client from app state
    let vpn_client = app_handle.state::<Arc<Mutex<vpn_client::VpnClient>>>();
    let mut client = vpn_client.lock().await;
    
    // Perform authentication
    let auth_manager = auth_manager::AuthManager::new();
    let cookie = auth_manager.authenticate(&portal_url).await
        .map_err(|e| format!("Authentication failed: {}", e))?;
    
    // Connect to VPN
    client.connect(&portal_url, gateway_address, cookie).await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    Ok("Connected successfully".to_string())
}

#[tauri::command]
async fn disconnect_vpn(app_handle: AppHandle) -> Result<String, String> {
    let vpn_client = app_handle.state::<Arc<Mutex<vpn_client::VpnClient>>>();
    let mut client = vpn_client.lock().await;
    
    client.disconnect().await
        .map_err(|e| format!("Disconnection failed: {}", e))?;
    
    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
async fn get_portals(app_handle: AppHandle) -> Result<Vec<portal_manager::Portal>, String> {
    let portal_manager = app_handle.state::<Arc<Mutex<portal_manager::PortalManager>>>();
    let manager = portal_manager.lock().await;
    Ok(manager.get_portals().clone())
}

#[tauri::command]
async fn add_portal(app_handle: AppHandle, portal: portal_manager::Portal) -> Result<(), String> {
    let portal_manager = app_handle.state::<Arc<Mutex<portal_manager::PortalManager>>>();
    let mut manager = portal_manager.lock().await;
    manager.add_portal(portal).await
        .map_err(|e| format!("Failed to add portal: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn remove_portal(app_handle: AppHandle, portal_id: String) -> Result<(), String> {
    let portal_manager = app_handle.state::<Arc<Mutex<portal_manager::PortalManager>>>();
    let mut manager = portal_manager.lock(). await;
    manager.remove_portal(&portal_id).await
        .map_err(|e| format!("Failed to remove portal: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn update_portal(app_handle: AppHandle, portal: portal_manager::Portal) -> Result<(), String> {
    let portal_manager = app_handle.state::<Arc<Mutex<portal_manager::PortalManager>>>();
    let mut manager = portal_manager.lock().await;
    manager.update_portal(portal).await
        .map_err(|e| format!("Failed to update portal: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn get_current_status(app_handle: AppHandle) -> Result<vpn_client::VpnStatus, String> {
    let vpn_client = app_handle.state::<Arc<Mutex<vpn_client::VpnClient>>>();
    let client = vpn_client.lock().await;
    Ok(client.get_status().clone())
}

#[tauri::command]
async fn enable_auto_connect(app_handle: AppHandle) -> Result<(), String> {
    let config = app_handle.state::<Arc<Mutex<config::Config>>>();
    let mut cfg = config.lock().await;
    cfg.auto_connect = true;
    cfg.save().await
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn disable_auto_connect(app_handle: AppHandle) -> Result<(), String> {
    let config = app_handle.state::<Arc<Mutex<config::Config>>>();
    let mut cfg = config.lock().await;
    cfg.auto_connect = false;
    cfg.save().await
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn open_settings(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        tauri::WindowBuilder::new(&app_handle, "main", tauri::WindowUrl::App("/".into()))
            .title("GlobalProtect Settings")
            .build()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    app_handle.exit(0);
    Ok(())
}