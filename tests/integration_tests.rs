// Integration tests for Desk Share Net
use desk_share_net::{AppState, Device, error::*};
use tokio;

#[tokio::test]
async fn test_app_state_initialization() {
    // Test that app state can be created
    let app_state = create_test_app_state().await;
    
    let user_name = app_state.user_name.lock().await;
    assert_eq!(*user_name, String::new());
}

#[tokio::test]
async fn test_device_serialization() {
    let device = Device {
        name: "Test Device".to_string(),
        ip: "192.168.1.100".to_string(),
        port: 8080,
        is_online: true,
        last_seen: "2026-01-28T16:00:00Z".to_string(),
    };
    
    let json = serde_json::to_string(&device).unwrap();
    assert!(json.contains("Test Device"));
    assert!(json.contains("192.168.1.100"));
}

// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    AppState {
        user_name: Arc::new(Mutex::new(String::new())),
        network_discovery: Arc::new(Mutex::new(
            desk_share_net::network::NetworkDiscovery::new().await
        )),
        file_transfer: Arc::new(Mutex::new(
            desk_share_net::network::FileTransfer::new().await
        )),
        screen_share: Arc::new(Mutex::new(
            desk_share_net::network::ScreenShare::new().await
        )),
        connected_devices: Arc::new(Mutex::new(Vec::new())),
    }
}
