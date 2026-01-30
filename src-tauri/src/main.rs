#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

// Import from the main application
use desk_share_net::{
    network::{NetworkDiscovery, FileTransfer, ScreenShare},
    AppState, Device,
};

// Tauri-specific state wrapper
struct TauriAppState {
    app_state: Arc<Mutex<AppState>>,
}

// ============================================================================
// Command Handlers
// ============================================================================

#[tauri::command]
async fn set_user_name(
    name: String,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let app_state = state.app_state.lock().await;
    let mut user_name = app_state.user_name.lock().await;
    *user_name = name.clone();
    
    tracing::info!("User name set to: {}", name);
    Ok(format!("User name set to: {}", name))
}

#[tauri::command]
async fn get_devices(
    state: State<'_, TauriAppState>,
) -> Result<Vec<Device>, String> {
    let app_state = state.app_state.lock().await;
    let discovery = app_state.network_discovery.lock().await;
    
    let devices = discovery.get_devices();
    tracing::debug!("Retrieved {} devices", devices.len());
    
    Ok(devices)
}

#[tauri::command]
async fn refresh_devices(
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let app_state = state.app_state.lock().await;
    let mut discovery = app_state.network_discovery.lock().await;
    
    discovery.cleanup_old_devices(300); // 5 minutes timeout
    tracing::info!("Devices refreshed");
    
    Ok("Devices refreshed".to_string())
}

#[derive(Serialize, Deserialize)]
struct FileTransferRequest {
    device_ip: String,
    file_path: String,
}

#[tauri::command]
async fn start_file_transfer(
    device_ip: String,
    file_path: String,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let app_state = state.app_state.lock().await;
    let file_transfer = app_state.file_transfer.lock().await;
    
    tracing::info!("Starting file transfer to {} for file: {}", device_ip, file_path);
    
    // In a real implementation, this would initiate the transfer
    // For now, we'll return a success message
    Ok(format!("File transfer started to {}", device_ip))
}

#[derive(Serialize, Deserialize)]
struct TransferProgress {
    file_name: String,
    percentage: f64,
    bytes_transferred: u64,
    total_bytes: u64,
}

#[tauri::command]
async fn get_transfer_progress(
    state: State<'_, TauriAppState>,
) -> Result<Vec<TransferProgress>, String> {
    // In a real implementation, this would track actual progress
    // For now, return empty array
    Ok(vec![])
}

#[derive(Serialize, Deserialize)]
struct ScreenShareRequest {
    frame_rate: u32,
}

#[tauri::command]
async fn start_screen_share(
    frame_rate: u32,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let app_state = state.app_state.lock().await;
    let screen_share = app_state.screen_share.lock().await;
    
    tracing::info!("Starting screen share with frame rate: {}", frame_rate);
    
    // Generate a session ID
    let session_id = format!("session_{}", chrono::Utc::now().timestamp());
    
    Ok(session_id)
}

#[tauri::command]
async fn stop_screen_share(
    session_id: String,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    tracing::info!("Stopping screen share session: {}", session_id);
    Ok("Screen share stopped".to_string())
}

#[tauri::command]
async fn join_screen_share(
    host_ip: String,
    host_port: u16,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    tracing::info!("Joining screen share at {}:{}", host_ip, host_port);
    Ok(format!("Joined screen share at {}:{}", host_ip, host_port))
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    from: String,
    to: Option<String>,
    message: String,
    timestamp: i64,
}

#[tauri::command]
async fn send_chat_message(
    message: String,
    to: Option<String>,
    state: State<'_, TauriAppState>,
) -> Result<String, String> {
    let app_state = state.app_state.lock().await;
    let user_name = app_state.user_name.lock().await;
    
    tracing::info!("Sending chat message from {}: {}", user_name, message);
    
    Ok("Message sent".to_string())
}

#[tauri::command]
async fn get_chat_history(
    state: State<'_, TauriAppState>,
) -> Result<Vec<ChatMessage>, String> {
    // In a real implementation, this would retrieve chat history
    Ok(vec![])
}

// ============================================================================
// Main Application
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting Desk Share Net application");

    // Initialize application state
    let app_state = AppState {
        user_name: Arc::new(Mutex::new(String::new())),
        network_discovery: Arc::new(Mutex::new(NetworkDiscovery::new().await)),
        file_transfer: Arc::new(Mutex::new(FileTransfer::new().await)),
        screen_share: Arc::new(Mutex::new(ScreenShare::new().await)),
        connected_devices: Arc::new(Mutex::new(Vec::new())),
    };

    // Start network discovery in background
    {
        let discovery = app_state.network_discovery.clone();
        tokio::spawn(async move {
            let mut discovery = discovery.lock().await;
            discovery.start_discovery().await;
            discovery.listen_for_devices().await;
        });
    }

    // Wrap state for Tauri
    let tauri_state = TauriAppState {
        app_state: Arc::new(Mutex::new(app_state)),
    };

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(tauri_state)
        .invoke_handler(tauri::generate_handler![
            set_user_name,
            get_devices,
            refresh_devices,
            start_file_transfer,
            get_transfer_progress,
            start_screen_share,
            stop_screen_share,
            join_screen_share,
            send_chat_message,
            get_chat_history,
        ])
        .setup(|app| {
            tracing::info!("Tauri application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
