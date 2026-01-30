use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate_type: CandidateType,
    pub address: String,
    pub port: u16,
    pub protocol: TransportProtocol,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CandidateType {
    Host,
    Srflx, // Server reflexive
    Relay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    UDP,
    TCP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServer {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub struct NatTraversal {
    stun_servers: Vec<StunServer>,
    turn_servers: Vec<TurnServer>,
    local_ip: IpAddr,
    socket: Option<UdpSocket>,
}

impl NatTraversal {
    pub async fn new() -> Result<Self, Error> {
        let local_ip = local_ip_address::local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
        
        Ok(Self {
            stun_servers: vec![
                StunServer { address: "stun.l.google.com".to_string(), port: 19302 },
                StunServer { address: "stun1.l.google.com".to_string(), port: 19302 },
                StunServer { address: "stun2.l.google.com".to_string(), port: 19302 },
            ],
            turn_servers: vec![], // Can be configured
            local_ip,
            socket: None,
        })
    }
    
    /// Add custom STUN servers
    pub fn add_stun_server(&mut self, address: String, port: u16) {
        self.stun_servers.push(StunServer { address, port });
    }
    
    /// Add TURN server for relay
    pub fn add_turn_server(&mut self, address: String, port: u16, username: String, password: String) {
        self.turn_servers.push(TurnServer { address, port, username, password });
    }
    
    /// Get local ICE candidates
    pub async fn get_local_candidates(&mut self) -> Result<Vec<IceCandidate>, Error> {
        let mut candidates = Vec::new();
        
        // Host candidate (local IP)
        candidates.push(IceCandidate {
            candidate_type: CandidateType::Host,
            address: self.local_ip.to_string(),
            port: 0, // Will be assigned when socket is bound
            protocol: TransportProtocol::UDP,
            priority: 2130706431, // High priority for local
        });
        
        // Server reflexive candidates (via STUN)
        for stun_server in &self.stun_servers {
            if let Ok(candidate) = self.get_stun_candidate(stun_server).await {
                candidates.push(candidate);
            }
        }
        
        // Relay candidates (via TURN)
        for turn_server in &self.turn_servers {
            if let Ok(candidate) = self.get_turn_candidate(turn_server).await {
                candidates.push(candidate);
            }
        }
        
        Ok(candidates)
    }
    
    /// Get server reflexive candidate using STUN
    async fn get_stun_candidate(&self, stun_server: &StunServer) -> Result<IceCandidate, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        // Create STUN binding request
        let stun_request = self.create_stun_binding_request();
        
        // Send to STUN server
        let addr = format!("{}:{}", stun_server.address, stun_server.port);
        socket.send_to(&stun_request, addr).await?;
        
        // Receive response
        let mut buf = [0u8; 1024];
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        
        match socket.recv_from(&mut buf).await {
            Ok((len, _)) => {
                if let Some((mapped_ip, mapped_port)) = self.parse_stun_response(&buf[..len]) {
                    return Ok(IceCandidate {
                        candidate_type: CandidateType::Srflx,
                        address: mapped_ip.to_string(),
                        port: mapped_port,
                        protocol: TransportProtocol::UDP,
                        priority: 1694498815, // Lower priority than host
                    });
                }
            }
            Err(_) => {
                // STUN failed, return error
                return Err(anyhow::anyhow!("STUN request failed"));
            }
        }
        
        Err(anyhow::anyhow!("Failed to get STUN candidate"))
    }
    
    /// Get relay candidate using TURN
    async fn get_turn_candidate(&self, turn_server: &TurnServer) -> Result<IceCandidate, Error> {
        // TURN allocation would require authentication
        // This is a simplified implementation
        Ok(IceCandidate {
            candidate_type: CandidateType::Relay,
            address: turn_server.address.clone(),
            port: turn_server.port,
            protocol: TransportProtocol::UDP,
            priority: 0, // Lowest priority
        })
    }
    
    /// Create STUN binding request
    fn create_stun_binding_request(&self) -> Vec<u8> {
        let mut request = Vec::new();
        
        // STUN message type (Binding Request) = 0x0001
        request.extend_from_slice(&[0x00, 0x01]);
        
        // Message length (0 for binding request)
        request.extend_from_slice(&[0x00, 0x00]);
        
        // Magic cookie
        request.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);
        
        // Transaction ID (12 bytes of random data)
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut transaction_id = [0u8; 12];
        rng.fill(&mut transaction_id);
        request.extend_from_slice(&transaction_id);
        
        request
    }
    
    /// Parse STUN response to extract mapped address
    fn parse_stun_response(&self, data: &[u8]) -> Option<(IpAddr, u16)> {
        if data.len() < 20 {
            return None;
        }
        
        // Check message type (should be 0x0101 for Binding Response)
        if data[0] != 0x01 || data[1] != 0x01 {
            return None;
        }
        
        // Parse attributes
        let mut offset = 20;
        while offset + 4 <= data.len() {
            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            
            offset += 4;
            
            if offset + attr_length as usize > data.len() {
                break;
            }
            
            // XOR-MAPPED-ADDRESS (0x0020)
            if attr_type == 0x0020 {
                if attr_length >= 8 {
                    let family = data[offset];
                    let port = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                    
                    if family == 0x01 { // IPv4
                        if attr_length >= 12 {
                            let ip_bytes = [
                                data[offset + 4] ^ 0x21,
                                data[offset + 5] ^ 0x12,
                                data[offset + 6] ^ 0xA4,
                                data[offset + 7] ^ 0x42,
                            ];
                            let ip = IpAddr::from(ip_bytes);
                            let port_fixed = port ^ 0x2112; // XOR with magic cookie first 2 bytes
                            return Some((ip, port_fixed));
                        }
                    } else if family == 0x02 { // IPv6
                        if attr_length >= 20 {
                            let mut ip_bytes = [0u8; 16];
                            for i in 0..16 {
                                ip_bytes[i] = data[offset + 4 + i] ^ [0x21, 0x12, 0xA4, 0x42][i % 4];
                            }
                            let ip = IpAddr::from(ip_bytes);
                            let port_fixed = port ^ 0x2112;
                            return Some((ip, port_fixed));
                        }
                    }
                }
            }
            
            offset += attr_length as usize;
        }
        
        None
    }
    
    /// Perform connectivity check
    pub async fn connectivity_check(&self, remote_candidate: &IceCandidate) -> Result<bool, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        
        // Send STUN binding request to remote candidate
        let stun_request = self.create_stun_binding_request();
        let addr = format!("{}:{}", remote_candidate.address, remote_candidate.port);
        
        socket.send_to(&stun_request, addr).await?;
        
        // Wait for response
        let mut buf = [0u8; 1024];
        socket.set_read_timeout(Some(Duration::from_secs(3)))?;
        
        match socket.recv_from(&mut buf).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Get relayed address from TURN server
    pub async fn allocate_relay(&self, turn_server: &TurnServer) -> Result<SocketAddr, Error> {
        // This would implement TURN allocation
        // For now, return the TURN server address
        Ok(SocketAddr::new(
            turn_server.address.parse()?,
            turn_server.port,
        ))
    }
}