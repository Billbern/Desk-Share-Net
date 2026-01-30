use thiserror::Error;
use std::io;

/// Main error type for Desk Share Net application
#[derive(Error, Debug)]
pub enum DeskShareError {
    // Network errors
    #[error("Network connection failed: {0}")]
    NetworkConnection(String),
    
    #[error("Device discovery failed: {0}")]
    DiscoveryFailed(String),
    
    #[error("NAT traversal failed: {0}")]
    NatTraversalFailed(String),
    
    #[error("Peer connection failed: {0}")]
    PeerConnectionFailed(String),
    
    // File transfer errors
    #[error("File transfer failed: {0}")]
    FileTransferFailed(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("File read error: {0}")]
    FileReadError(#[from] io::Error),
    
    #[error("Chunk transfer failed: {0}")]
    ChunkTransferFailed(String),
    
    #[error("File integrity check failed")]
    IntegrityCheckFailed,
    
    // Screen sharing errors
    #[error("Screen capture failed: {0}")]
    ScreenCaptureFailed(String),
    
    #[error("Screen share session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Video encoding failed: {0}")]
    EncodingFailed(String),
    
    // WebRTC signaling errors
    #[error("Signaling failed: {0}")]
    SignalingFailed(String),
    
    #[error("SDP offer/answer exchange failed: {0}")]
    SdpExchangeFailed(String),
    
    #[error("ICE candidate exchange failed: {0}")]
    IceCandidateFailed(String),
    
    // Chat errors
    #[error("Message send failed: {0}")]
    MessageSendFailed(String),
    
    #[error("Invalid message format")]
    InvalidMessageFormat,
    
    // General errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Operation timeout")]
    Timeout,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for Desk Share Net operations
pub type Result<T> = std::result::Result<T, DeskShareError>;

/// Error recovery strategies
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry { max_attempts: u32, backoff_ms: u64 },
    /// Fallback to alternative method
    Fallback,
    /// Fail immediately
    Fail,
}

impl DeskShareError {
    /// Determine the appropriate recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // Network errors - retry with backoff
            DeskShareError::NetworkConnection(_) 
            | DeskShareError::PeerConnectionFailed(_) 
            | DeskShareError::ChunkTransferFailed(_) => {
                RecoveryStrategy::Retry {
                    max_attempts: 3,
                    backoff_ms: 1000,
                }
            }
            
            // NAT traversal - fallback to TURN relay
            DeskShareError::NatTraversalFailed(_) => RecoveryStrategy::Fallback,
            
            // File errors - fail immediately
            DeskShareError::FileNotFound(_) 
            | DeskShareError::IntegrityCheckFailed => RecoveryStrategy::Fail,
            
            // Screen capture - retry once
            DeskShareError::ScreenCaptureFailed(_) => {
                RecoveryStrategy::Retry {
                    max_attempts: 1,
                    backoff_ms: 500,
                }
            }
            
            // Default - fail
            _ => RecoveryStrategy::Fail,
        }
    }
    
    /// Convert error to user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            DeskShareError::NetworkConnection(_) => {
                "Unable to connect to the network. Please check your connection.".to_string()
            }
            DeskShareError::FileNotFound(path) => {
                format!("File not found: {}", path)
            }
            DeskShareError::ScreenCaptureFailed(_) => {
                "Failed to capture screen. Please check permissions.".to_string()
            }
            DeskShareError::PeerConnectionFailed(_) => {
                "Failed to connect to peer. They may be offline.".to_string()
            }
            DeskShareError::Timeout => {
                "Operation timed out. Please try again.".to_string()
            }
            _ => self.to_string(),
        }
    }
}

/// Retry helper with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_backoff_ms: u64,
) -> std::result::Result<T, E>
where
    F: FnMut() -> std::result::Result<T, E>,
{
    let mut attempts = 0;
    let mut backoff = initial_backoff_ms;
    
    loop {
        attempts += 1;
        
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;
                backoff *= 2; // Exponential backoff
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_recovery_strategy() {
        let error = DeskShareError::NetworkConnection("test".to_string());
        match error.recovery_strategy() {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("Expected retry strategy"),
        }
    }
    
    #[test]
    fn test_user_message() {
        let error = DeskShareError::FileNotFound("/test/file.txt".to_string());
        let message = error.user_message();
        assert!(message.contains("File not found"));
    }
}
