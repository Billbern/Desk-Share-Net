use libp2p::request_response::{
    Codec, Behaviour, ProtocolSupport,
};
use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::error::{DeskShareError, Result};

/// WebRTC signaling messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    /// SDP Offer
    Offer {
        from: String,
        to: String,
        sdp: String,
    },
    /// SDP Answer
    Answer {
        from: String,
        to: String,
        sdp: String,
    },
    /// ICE Candidate
    IceCandidate {
        from: String,
        to: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    },
    /// Connection request
    ConnectRequest {
        from: String,
        to: String,
    },
    /// Connection accepted
    ConnectAccept {
        from: String,
        to: String,
    },
    /// Connection rejected
    ConnectReject {
        from: String,
        to: String,
        reason: String,
    },
}

/// Signaling protocol codec for libp2p
#[derive(Debug, Clone, Default)]
pub struct SignalingCodec;

impl AsRef<str> for SignalingCodec {
    fn as_ref(&self) -> &str {
        "/webrtc-signaling/1.0.0"
    }
}

#[async_trait]
impl Codec for SignalingCodec {
    type Protocol = SignalingCodec;
    type Request = SignalingMessage;
    type Response = SignalingMessage;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        
        serde_json::from_slice(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        io.write_all(&data).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&res)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        io.write_all(&data).await?;
        io.close().await
    }
}

/// Signaling server for WebRTC connections
pub struct SignalingServer {
    /// Request-response behavior for libp2p
    behaviour: Behaviour<SignalingCodec>,
    
    /// Pending signaling messages
    pending_messages: Arc<RwLock<HashMap<String, Vec<SignalingMessage>>>>,
    
    /// Message broadcast channel
    message_tx: mpsc::Sender<SignalingMessage>,
    message_rx: Arc<RwLock<mpsc::Receiver<SignalingMessage>>>,
    
    /// Local peer ID
    local_peer_id: String,
}

impl SignalingServer {
    pub fn new(local_peer_id: String) -> Self {
        let behaviour = Behaviour::new(
            std::iter::once((SignalingCodec, ProtocolSupport::Full)),
            libp2p::request_response::Config::default(),
        );
        
        let (message_tx, message_rx) = mpsc::channel(100);
        
        Self {
            behaviour,
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(RwLock::new(message_rx)),
            local_peer_id,
        }
    }
    
    /// Send an offer to a peer
    pub async fn send_offer(
        &self,
        to: String,
        sdp: String,
    ) -> Result<()> {
        let message = SignalingMessage::Offer {
            from: self.local_peer_id.clone(),
            to: to.clone(),
            sdp,
        };
        
        self.send_message(to, message).await
    }
    
    /// Send an answer to a peer
    pub async fn send_answer(
        &self,
        to: String,
        sdp: String,
    ) -> Result<()> {
        let message = SignalingMessage::Answer {
            from: self.local_peer_id.clone(),
            to: to.clone(),
            sdp,
        };
        
        self.send_message(to, message).await
    }
    
    /// Send an ICE candidate to a peer
    pub async fn send_ice_candidate(
        &self,
        to: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
    ) -> Result<()> {
        let message = SignalingMessage::IceCandidate {
            from: self.local_peer_id.clone(),
            to: to.clone(),
            candidate,
            sdp_mid,
            sdp_mline_index,
        };
        
        self.send_message(to, message).await
    }
    
    /// Send a connection request
    pub async fn request_connection(&self, to: String) -> Result<()> {
        let message = SignalingMessage::ConnectRequest {
            from: self.local_peer_id.clone(),
            to: to.clone(),
        };
        
        self.send_message(to, message).await
    }
    
    /// Accept a connection request
    pub async fn accept_connection(&self, to: String) -> Result<()> {
        let message = SignalingMessage::ConnectAccept {
            from: self.local_peer_id.clone(),
            to: to.clone(),
        };
        
        self.send_message(to, message).await
    }
    
    /// Reject a connection request
    pub async fn reject_connection(&self, to: String, reason: String) -> Result<()> {
        let message = SignalingMessage::ConnectReject {
            from: self.local_peer_id.clone(),
            to: to.clone(),
            reason,
        };
        
        self.send_message(to, message).await
    }
    
    /// Send a signaling message
    async fn send_message(&self, to: String, message: SignalingMessage) -> Result<()> {
        // Store in pending messages
        let mut pending = self.pending_messages.write().await;
        pending
            .entry(to.clone())
            .or_insert_with(Vec::new)
            .push(message.clone());
        
        // Broadcast via channel
        self.message_tx
            .send(message)
            .await
            .map_err(|e| DeskShareError::SignalingFailed(e.to_string()))?;
        
        tracing::debug!("Signaling message sent to {}", to);
        
        Ok(())
    }
    
    /// Receive a signaling message
    pub async fn receive_message(&self, message: SignalingMessage) -> Result<()> {
        tracing::debug!("Signaling message received: {:?}", message);
        
        // Broadcast to listeners
        self.message_tx
            .send(message)
            .await
            .map_err(|e| DeskShareError::SignalingFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get pending messages for a peer
    pub async fn get_pending_messages(&self, peer_id: &str) -> Vec<SignalingMessage> {
        let mut pending = self.pending_messages.write().await;
        pending.remove(peer_id).unwrap_or_default()
    }
    
    /// Subscribe to signaling messages
    pub async fn subscribe(&self) -> mpsc::Receiver<SignalingMessage> {
        let (tx, rx) = mpsc::channel(100);
        
        // Clone the receiver (this is a simplified version)
        // In production, you'd use a broadcast channel
        rx
    }
}

/// Helper to create WebRTC offer
pub async fn create_offer() -> Result<RTCSessionDescription> {
    // This would use the actual WebRTC API
    // For now, return a placeholder
    Err(DeskShareError::SignalingFailed("Not implemented".to_string()))
}

/// Helper to create WebRTC answer
pub async fn create_answer(offer: RTCSessionDescription) -> Result<RTCSessionDescription> {
    // This would use the actual WebRTC API
    // For now, return a placeholder
    Err(DeskShareError::SignalingFailed("Not implemented".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_signaling_server() {
        let server = SignalingServer::new("peer1".to_string());
        
        let result = server
            .send_offer("peer2".to_string(), "sdp_offer".to_string())
            .await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_ice_candidate() {
        let server = SignalingServer::new("peer1".to_string());
        
        let result = server
            .send_ice_candidate(
                "peer2".to_string(),
                "candidate".to_string(),
                Some("0".to_string()),
                Some(0),
            )
            .await;
        
        assert!(result.is_ok());
    }
}
