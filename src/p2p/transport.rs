// P2P transport layer implementation
// Handles data transfer between peers

use tokio::sync::mpsc;
use bytes::Bytes;
use std::collections::HashMap;

pub struct P2PTransport {
    connections: HashMap<String, Connection>,
}

pub struct Connection {
    peer_id: String,
    sender: mpsc::Sender<Bytes>,
    receiver: mpsc::Receiver<Bytes>,
}

impl P2PTransport {
    pub fn new() -> Self {
        P2PTransport {
            connections: HashMap::new(),
        }
    }
    
    pub async fn connect(&mut self, peer_id: String) -> Result<(), String> {
        let (tx, rx) = mpsc::channel(1000);
        let (return_tx, return_rx) = mpsc::channel(1000);
        
        let connection = Connection {
            peer_id: peer_id.clone(),
            sender: tx,
            receiver: return_rx,
        };
        
        self.connections.insert(peer_id.clone(), connection);
        tracing::info!("Connected to peer: {}", peer_id);
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self, peer_id: &str) {
        self.connections.remove(peer_id);
        tracing::info!("Disconnected from peer: {}", peer_id);
    }
    
    pub async fn send(&self, peer_id: &str, data: Bytes) -> Result<(), String> {
        if let Some(conn) = self.connections.get(peer_id) {
            conn.sender.send(data).await
                .map_err(|e| format!("Failed to send data: {}", e))?;
            Ok(())
        } else {
            Err(format!("No connection to peer: {}", peer_id))
        }
    }
    
    pub async fn receive(&mut self, peer_id: &str) -> Option<Bytes> {
        if let Some(conn) = self.connections.get_mut(peer_id) {
            conn.receiver.recv().await
        } else {
            None
        }
    }
    
    pub fn is_connected(&self, peer_id: &str) -> bool {
        self.connections.contains_key(peer_id)
    }
}

impl Connection {
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }
}