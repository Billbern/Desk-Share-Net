use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Error;

pub struct ScreenShare {
    sessions: Arc<RwLock<HashMap<String, SharingSession>>>,
    frame_buffer: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    capture_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Clone)]
pub struct SharingSession {
    pub session_id: String,
    pub host_peer_id: String,
    pub participants: HashSet<String>,
    pub is_recording: bool,
    pub frame_rate: u32,
    pub resolution: (u32, u32),
    pub codec: String,
}

impl ScreenShare {
    pub async fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            frame_buffer: Arc::new(RwLock::new(HashMap::new())),
            capture_handle: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn start_sharing(
        &self,
        peer_id: String,
        frame_rate: u32,
        resolution: (u32, u32),
    ) -> Result<String, Error> {
        let session_id = Self::generate_session_id();
        
        let session = SharingSession {
            session_id: session_id.clone(),
            host_peer_id: peer_id.clone(),
            participants: HashSet::new(),
            is_recording: true,
            frame_rate,
            resolution,
            codec: "VP8".to_string(),
        };
        
        self.sessions.write().await.insert(session_id.clone(), session);
        
        // Start screen capture
        self.start_screen_capture(&session_id, frame_rate, resolution).await?;
        
        // Announce session to network
        self.announce_session(&session_id, peer_id).await?;
        
        Ok(session_id)
    }
    
    pub async fn join_session(&self, session_id: &str, peer_id: String) -> Result<(), Error> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.participants.insert(peer_id.clone());
            
            // Request video stream from host
            self.request_video_stream(session_id, peer_id, session.host_peer_id.clone()).await?;
        }
        
        Ok(())
    }
    
    pub async fn leave_session(&self, session_id: &str, peer_id: String) -> Result<(), Error> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.participants.remove(&peer_id);
        }
        
        Ok(())
    }
    
    pub async fn stop_sharing(&self, session_id: &str) -> Result<(), Error> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get(session_id) {
            if session.host_peer_id == session_id {
                // Stop the capture task
                let mut handle = self.capture_handle.write().await;
                if let Some(h) = handle.take() {
                    h.abort();
                }
                
                sessions.remove(session_id);
            }
        }
        
        Ok(())
    }
    
    pub async fn broadcast_to_session(&self, session_id: &str, frame_data: &[u8]) -> Result<(), Error> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            // Store frame in buffer
            self.frame_buffer.write().await.insert(
                format!("{}-latest", session_id),
                frame_data.to_vec(),
            );
            
            // Send to all participants (mesh distribution)
            for participant in &session.participants {
                self.send_frame_to_peer(participant, frame_data).await?;
            }
        }
        
        Ok(())
    }
    
    pub async fn get_frame(&self, session_id: &str) -> Option<Vec<u8>> {
        let buffer = self.frame_buffer.read().await;
        buffer.get(&format!("{}-latest", session_id)).cloned()
    }
    
    async fn start_screen_capture(
        &self,
        session_id: &str,
        frame_rate: u32,
        resolution: (u32, u32),
    ) -> Result<(), Error> {
        let session_id = session_id.to_string();
        let frame_buffer = self.frame_buffer.clone();
        let sessions = self.sessions.clone();
        
        let handle = tokio::spawn(async move {
            let frame_interval = std::time::Duration::from_millis(1000 / frame_rate as u64);
            
            loop {
                // Check if session is still active
                let session_exists = {
                    let sessions = sessions.read().await;
                    sessions.contains_key(&session_id)
                };
                
                if !session_exists {
                    break;
                }
                
                // Capture screen (platform-specific implementation)
                let frame = Self::capture_screen_frame(resolution).await;
                
                // Store in buffer
                frame_buffer.write().await.insert(
                    format!("{}-latest", session_id),
                    frame.clone(),
                );
                
                // Broadcast to participants
                if let Some(session) = sessions.read().await.get(&session_id) {
                    for participant in &session.participants {
                        // Send frame to participant
                        // This would use P2P transport
                        let _ = Self::send_frame_to_peer_static(participant, &frame).await;
                    }
                }
                
                tokio::time::sleep(frame_interval).await;
            }
        });
        
        *self.capture_handle.write().await = Some(handle);
        
        Ok(())
    }
    
    async fn capture_screen_frame(resolution: (u32, u32)) -> Vec<u8> {
        // Use platform-specific screen capture
        match crate::platform::capture_screen(resolution).await {
            Ok(frame) => frame,
            Err(e) => {
                tracing::error!("Screen capture failed: {}", e);
                // Fallback to test pattern
                Self::generate_test_pattern(resolution)
            }
        }
    }
    
    fn generate_test_pattern(resolution: (u32, u32)) -> Vec<u8> {
        // Generate a simple test pattern for demonstration
        let (width, height) = resolution;
        let mut data = Vec::with_capacity((width * height * 3) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
                data.push(r);
                data.push(g);
                data.push(b);
            }
        }
        
        data
    }
    
    async fn send_frame_to_peer(&self, peer_id: &str, frame_data: &[u8]) -> Result<(), Error> {
        // This would use the P2P transport to send frame data
        // Implementation depends on the transport layer
        Ok(())
    }
    
    async fn send_frame_to_peer_static(peer_id: &str, frame_data: &[u8]) -> Result<(), Error> {
        // Static version for use in spawn
        // This would use the P2P transport
        Ok(())
    }
    
    async fn request_video_stream(&self, session_id: &str, peer_id: String, host_peer_id: String) -> Result<(), Error> {
        // Request video stream from host
        // This would use WebRTC or custom protocol
        Ok(())
    }
    
    async fn announce_session(&self, session_id: &str, host_peer_id: String) -> Result<(), Error> {
        let announcement = serde_json::to_vec(&SessionAnnouncement {
            session_id: session_id.to_string(),
            host_peer_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })?;
        
        // Broadcast announcement through P2P network
        // This would use the network broadcast mechanism
        Ok(())
    }
    
    fn generate_session_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:x}", rng.gen::<u128>())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SessionAnnouncement {
    session_id: String,
    host_peer_id: String,
    timestamp: u64,
}