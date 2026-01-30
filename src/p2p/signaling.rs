// WebRTC signaling server and client implementation
// Handles peer connection establishment and ICE candidate exchange

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    Offer { sdp: String, from: String, to: String },
    Answer { sdp: String, from: String, to: String },
    IceCandidate { candidate: String, from: String, to: String },
    Join { peer_id: String },
    Leave { peer_id: String },
}

pub struct SignalingServer {
    peers: HashMap<String, mpsc::Sender<SignalingMessage>>,
}

impl SignalingServer {
    pub fn new() -> Self {
        SignalingServer {
            peers: HashMap::new(),
        }
    }
    
    pub async fn register_peer(&mut self, peer_id: String) -> mpsc::Receiver<SignalingMessage> {
        let (tx, rx) = mpsc::channel(100);
        self.peers.insert(peer_id.clone(), tx);
        tracing::info!("Peer registered: {}", peer_id);
        rx
    }
    
    pub async fn unregister_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
        tracing::info!("Peer unregistered: {}", peer_id);
    }
    
    pub async fn relay_message(&self, msg: SignalingMessage) -> Result<(), String> {
        let to = match &msg {
            SignalingMessage::Offer { to, .. } => to,
            SignalingMessage::Answer { to, .. } => to,
            SignalingMessage::IceCandidate { to, .. } => to,
            _ => return Ok(()),
        };
        
        if let Some(sender) = self.peers.get(to) {
            sender.send(msg).await
                .map_err(|e| format!("Failed to relay message: {}", e))?;
        }
        
        Ok(())
    }
}

pub struct SignalingClient {
    peer_id: String,
    receiver: Option<mpsc::Receiver<SignalingMessage>>,
}

impl SignalingClient {
    pub fn new(peer_id: String, receiver: mpsc::Receiver<SignalingMessage>) -> Self {
        SignalingClient {
            peer_id,
            receiver: Some(receiver),
        }
    }
    
    pub async fn receive_message(&mut self) -> Option<SignalingMessage> {
        if let Some(ref mut rx) = self.receiver {
            rx.recv().await
        } else {
            None
        }
    }
}
