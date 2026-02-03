use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime};
use local_ip_address::local_ip;
// use webrtc::mdns::{Record, RecordKind}; // TODO: Re-enable when implementing mDNS
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub services: Vec<String>,
    pub last_seen: u64,
}

pub struct NetworkDiscovery {
    devices: HashMap<String, DeviceInfo>,
    broadcast_sender: broadcast::Sender<DeviceInfo>,
    local_ip: IpAddr,
}

impl NetworkDiscovery {
    pub async fn new() -> Self {
        let local_ip = local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
        let (tx, _) = broadcast::channel(100);
        
        NetworkDiscovery {
            devices: HashMap::new(),
            broadcast_sender: tx,
            local_ip,
        }
    }
    
    pub async fn start_discovery(&mut self) {
        let local_ip = self.local_ip;
        let tx = self.broadcast_sender.clone();
        
        // TODO: Implement mDNS discovery with mdns crate v3.0
        // Currently using broadcast discovery only
        // Uncomment when ready to integrate mdns crate
        /*
        tokio::spawn(async move {
            let service = "_desktopshare._tcp.local";
            // mDNS v3.0 API differs from previous versions
            // Need to update to new API
        });
        */
        
        // Start broadcast discovery
        tokio::spawn(async move {
            let socket = UdpSocket::bind("0.0.0.0:5353").await.unwrap();
            socket.set_broadcast(true).unwrap();
            
            let broadcast_msg = format!("DISCOVER_DESKTOPSHARE:{}", local_ip);
            
            loop {
                let broadcast_addr = "255.255.255.255:5353";
                socket.send_to(broadcast_msg.as_bytes(), broadcast_addr).await.ok();
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }
    
    pub async fn listen_for_devices(&mut self) {
        let mut rx = self.broadcast_sender.subscribe();
        let socket = UdpSocket::bind("0.0.0.0:5353").await.unwrap();
        
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buf).await {
                    let msg = String::from_utf8_lossy(&buf[..len]);
                    if msg.starts_with("DISCOVER_DESKTOPSHARE:") {
                        let parts: Vec<&str> = msg.split(':').collect();
                        if parts.len() > 1 {
                            let device = DeviceInfo {
                                name: format!("Device-{}", parts[1]),
                                ip: parts[1].to_string(),
                                port: 8080,
                                services: vec!["file-transfer".to_string(), "screen-share".to_string()],
                                last_seen: SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            };
                            self.devices.insert(device.ip.clone(), device.clone());
                        }
                    }
                }
            }
        });
        
        tokio::spawn(async move {
            while let Ok(device) = rx.recv().await {
                self.devices.insert(device.ip.clone(), device);
            }
        });
    }
    
    pub fn get_devices(&self) -> Vec<DeviceInfo> {
        self.devices.values().cloned().collect()
    }
    
    pub fn cleanup_old_devices(&mut self, max_age_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        self.devices.retain(|_, device| {
            now - device.last_seen < max_age_seconds
        });
    }
}