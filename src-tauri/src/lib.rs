mod backend {
    pub mod config;
    pub mod hwinfo;
    pub mod mahm;
    pub mod pipe_client;
    pub mod process_manager;
}

use std::collections::HashMap;
use serde_json::Value;
use crate::backend::config::ConfigManager;
use crate::backend::pipe_client::PipeClient;
use crate::backend::process_manager::ProcessManager;
use crate::backend::hwinfo::{HwInfoReader, HwInfoData};
use crate::backend::mahm::{MahmReader, MahmData};
use tauri::Manager;

#[tauri::command]
fn get_settings(config: tauri::State<'_, ConfigManager>) -> HashMap<String, Value> {
    config.read_all()
}

#[tauri::command]
fn save_settings(config: tauri::State<'_, ConfigManager>, settings: HashMap<String, Value>) -> Result<(), String> {
    config.save_all(&settings)
}

#[tauri::command]
fn start_companion_process(app: tauri::AppHandle) -> Result<(), String> {
    ProcessManager::start_sidecar(&app)
}

#[tauri::command]
fn stop_companion_process() {
    ProcessManager::stop_sidecar()
}

#[tauri::command]
fn is_elevated() -> bool {
    ProcessManager::is_elevated()
}

#[tauri::command]
fn elevate() {
    ProcessManager::elevate_process()
}

#[tauri::command]
fn is_autostart_enabled() -> bool {
    ProcessManager::is_autostart_enabled()
}

#[tauri::command]
fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        ProcessManager::enable_autostart()
    } else {
        ProcessManager::disable_autostart()
    }
}

#[tauri::command]
fn install_service(app: tauri::AppHandle) -> Result<(), String> {
    ProcessManager::install_service(&app)
}

#[tauri::command]
fn uninstall_service() -> Result<(), String> {
    ProcessManager::uninstall_service()
}

#[tauri::command]
async fn select_present_mon_app(pipe: tauri::State<'_, PipeClient>, name: String) -> Result<(), String> {
    pipe.select_present_mon_app(name).await
}

#[tauri::command]
async fn select_polling_rate(pipe: tauri::State<'_, PipeClient>, interval: i16) -> Result<(), String> {
    pipe.select_polling_rate(interval).await
}

#[tauri::command]
async fn refresh_present_mon_apps(pipe: tauri::State<'_, PipeClient>) -> Result<(), String> {
    pipe.refresh_present_mon_apps().await
}

#[tauri::command]
fn read_hwinfo_data() -> Result<HwInfoData, String> {
    let mut reader = HwInfoReader::new();
    reader.read_data()
}

#[tauri::command]
fn read_mahm_data() -> Result<MahmData, String> {
    let mut reader = MahmReader::new();
    reader.read_data()
}

#[tauri::command]
fn set_window_click_through(window: tauri::Window, transparent: bool) -> Result<(), String> {
    window.set_ignore_cursor_events(transparent)
        .map_err(|e| format!("Failed to set click through: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Start the Named Pipe listener
            let pipe_client = PipeClient::new();
            pipe_client.start(app_handle);
            
            // Manage core state services
            app.manage(pipe_client);
            app.manage(ConfigManager::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            start_companion_process,
            stop_companion_process,
            is_elevated,
            elevate,
            is_autostart_enabled,
            set_autostart_enabled,
            install_service,
            uninstall_service,
            select_present_mon_app,
            select_polling_rate,
            refresh_present_mon_apps,
            read_hwinfo_data,
            read_mahm_data,
            set_window_click_through
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
