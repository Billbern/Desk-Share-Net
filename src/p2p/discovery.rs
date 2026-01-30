// Network discovery implementation
// Integrates the legacy discovery with new P2P structure

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use tokio::sync::broadcast;

use crate::app::Device;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub services: Vec<String>,
    pub last_seen: u64,
}

impl From<DeviceInfo> for Device {
    fn from(info: DeviceInfo) -> Self {
        Device {
            name: info.name,
            ip: info.ip,
            port: info.port,
            is_online: true,
            last_seen: chrono::DateTime::from_timestamp(info.last_seen as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
        }
    }
}

pub struct NetworkDiscovery {
    devices: HashMap<String, DeviceInfo>,
    broadcast_sender: broadcast::Sender<DeviceInfo>,
    local_ip: IpAddr,
}

impl NetworkDiscovery {
    pub async fn new() -> Self {
        let local_ip = local_ip_address::local_ip()
            .unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
        let (tx, _) = broadcast::channel(100);
        
        NetworkDiscovery {
            devices: HashMap::new(),
            broadcast_sender: tx,
            local_ip,
        }
    }
    
    pub async fn start_discovery(&mut self) {
        let local_ip = self.local_ip;
        
        tracing::info!("Starting network discovery on {}", local_ip);
        
        // Start mDNS discovery
        let tx = self.broadcast_sender.clone();
        tokio::spawn(async move {
            Self::mdns_discovery(tx).await;
        });
        
        // Start broadcast discovery
        tokio::spawn(async move {
            Self::broadcast_discovery(local_ip).await;
        });
    }
    
    async fn mdns_discovery(tx: broadcast::Sender<DeviceInfo>) {
        // mDNS discovery implementation
        // This would use the mdns crate to discover services
        tracing::debug!("mDNS discovery started");
    }
    
    async fn broadcast_discovery(local_ip: IpAddr) {
        // Broadcast discovery implementation
        tracing::debug!("Broadcast discovery started for {}", local_ip);
    }
    
    pub async fn listen_for_devices(&mut self) {
        tracing::info!("Listening for devices");
        // Device listening implementation
    }
    
    pub fn get_devices(&self) -> Vec<Device> {
        self.devices
            .values()
            .map(|info| info.clone().into())
            .collect()
    }
    
    pub fn cleanup_old_devices(&mut self, max_age_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        self.devices.retain(|_, device| {
            now - device.last_seen < max_age_seconds
        });
        
        tracing::debug!("Cleaned up old devices, {} remaining", self.devices.len());
    }
}