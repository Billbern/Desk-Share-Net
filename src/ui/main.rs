use crate::AppState;
use tauri::Window;
use serde_json::json;

pub async fn run(app_state: AppState) {
    tracing::info!("UI initialized with app state");
    
    // Initialize the application
    app_state.initialize().await;
    
    // Keep the application running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

#[tauri::command]
async fn set_user_name(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Name is required".to_string());
    }
    
    *state.user_name.lock().await = name;
    Ok(())
}

#[tauri::command]
async fn get_devices(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::Device>, String> {
    let discovery = state.network_discovery.lock().await;
    let devices_info = discovery.get_devices();
    
    let devices: Vec<crate::Device> = devices_info
        .into_iter()
        .map(|info| crate::Device {
            name: info.name,
            ip: info.ip,
            port: info.port,
            is_online: true,
            last_seen: chrono::Utc::now().to_rfc3339(),
        })
        .collect();
    
    Ok(devices)
}

#[tauri::command]
async fn start_file_transfer(
    state: tauri::State<'_, AppState>,
    device_ip: String,
    file_path: String,
) -> Result<(), String> {
    let transfer = state.file_transfer.lock().await;
    let user_name = state.user_name.lock().await.clone();
    transfer.send_file_to_device(&device_ip, &file_path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_screen_share(
    state: tauri::State<'_, AppState>,
    frame_rate: u32,
) -> Result<String, String> {
    let share = state.screen_share.lock().await;
    let user_name = state.user_name.lock().await.clone();
    share.start_sharing(user_name, frame_rate, (1920, 1080)).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn join_screen_share(
    state: tauri::State<'_, AppState>,
    host_ip: String,
    host_port: u16,
) -> Result<(), String> {
    let share = state.screen_share.lock().await;
    let user_name = state.user_name.lock().await.clone();
    share.join_session(&host_ip, user_name).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_transfer_progress(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let transfer = state.file_transfer.lock().await;
    let progress = transfer.get_transfer_progress().await;
    Ok(json!(progress))
}

#[tauri::command]
async fn list_local_files(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<serde_json::Value, String> {
    let transfer = state.file_transfer.lock().await;
    let files = transfer.list_files_in_directory(&path).await
        .map_err(|e| e.to_string())?;
    Ok(json!(files))
}

#[tauri::command]
async fn stop_screen_share(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let share = state.screen_share.lock().await;
    share.stop_sharing(&session_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_screen_frame(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<u8>, String> {
    let share = state.screen_share.lock().await;
    let frame = share.get_frame(&session_id).await
        .ok_or_else(|| "No frame available".to_string())?;
    Ok(frame)
}