mod network;
mod platform;
mod ui;
mod p2p;
mod services;
pub mod error;

pub use crate::p2p::NetworkDiscovery;
use crate::services::{FileTransfer, ScreenShare, ChatService};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub user_name: Arc<Mutex<String>>,
    pub network_discovery: Arc<Mutex<NetworkDiscovery>>,
    pub file_transfer: Arc<Mutex<FileTransfer>>,
    pub screen_share: Arc<Mutex<ScreenShare>>,
    pub connected_devices: Arc<Mutex<Vec<Device>>>,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            user_name: Arc::new(Mutex::new(String::new())),
            network_discovery: Arc::new(Mutex::new(NetworkDiscovery::new().await)),
            file_transfer: Arc::new(Mutex::new(FileTransfer::new().await)),
            screen_share: Arc::new(Mutex::new(ScreenShare::new().await)),
            connected_devices: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn initialize(&self) {
        // Start network discovery
        let mut discovery = self.network_discovery.lock().await;
        discovery.start_discovery().await;
        discovery.listen_for_devices().await;
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub is_online: bool,
    pub last_seen: String,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize application state
    let app_state = AppState::new().await;

    // Start the UI
    ui::run(app_state).await;
}