use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

use crate::p2p::NetworkDiscovery;
use crate::services::{FileTransfer, ScreenShare, ChatService};

/// Main application state shared across the application
#[derive(Clone)]
pub struct AppState {
    pub user_name: Arc<Mutex<String>>,
    pub network_discovery: Arc<Mutex<NetworkDiscovery>>,
    pub file_transfer: Arc<Mutex<FileTransfer>>,
    pub screen_share: Arc<Mutex<ScreenShare>>,
    pub chat_service: Arc<Mutex<ChatService>>,
    pub connected_devices: Arc<Mutex<Vec<Device>>>,
}

impl AppState {
    /// Create a new application state
    pub async fn new() -> Self {
        Self {
            user_name: Arc::new(Mutex::new(String::new())),
            network_discovery: Arc::new(Mutex::new(NetworkDiscovery::new().await)),
            file_transfer: Arc::new(Mutex::new(FileTransfer::new().await)),
            screen_share: Arc::new(Mutex::new(ScreenShare::new().await)),
            chat_service: Arc::new(Mutex::new(ChatService::new().await)),
            connected_devices: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize and start background services
    pub async fn initialize(&self) {
        // Start network discovery
        let discovery = self.network_discovery.clone();
        tokio::spawn(async move {
            let mut discovery = discovery.lock().await;
            discovery.start_discovery().await;
            discovery.listen_for_devices().await;
        });

        tracing::info!("Application state initialized");
    }
}

/// Represents a discovered device on the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub is_online: bool,
    pub last_seen: String,
}

impl Device {
    pub fn new(name: String, ip: String, port: u16) -> Self {
        Self {
            name,
            ip,
            port,
            is_online: true,
            last_seen: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now().to_rfc3339();
        self.is_online = true;
    }

    pub fn mark_offline(&mut self) {
        self.is_online = false;
    }
}
