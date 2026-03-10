use tauri::AppHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AutoConnectManager;

impl AutoConnectManager {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn enable_auto_start(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            use winreg::RegKey;
            use winreg::enums::*;
            
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let startup = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)?;
            startup.set_value("GlobalProtect-GUI", &r#"C:\Program Files\GlobalProtect\gpgui.exe"#)?;
        }
        
        #[cfg(target_os = "linux")]
        {
            // Create desktop entry for autostart
            let autostart_dir = dirs::config_dir()
                .ok_or("Could not determine config directory")?
                .join("autostart");
            std::fs::create_dir_all(&autostart_dir)?;
            
            let desktop_entry = r#"[Desktop Entry]
Type=Application
Name=GlobalProtect GUI
Exec=gpgui
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
"#;
            
            std std::fs::write(autostart_dir.join("globalprotect-gui.desktop"), desktop_entry)?;
        }
        
        Ok(())
    }
    
    pub async fn disable_auto_start(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            use winreg::RegKey;
            use winreg::enums::*;
            
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let startup = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)?;
            startup.delete_value("GlobalProtect-GUI")?;
        }
        
        #[cfg(target_os = "linux")]
        {
            let autostart_dir = dirs::config_dir()
                .ok_or("Could not determine config directory")?
                .join("autostart")
                .join("globalprotect-gui.desktop");
            if autostart_dir.exists() {
                std::fs::remove_file(autostart_dir)?;
            }
        }
        
        Ok(())
    }
}