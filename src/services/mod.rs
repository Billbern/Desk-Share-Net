// Services module
// Provides high-level services: file sharing, screen sharing, and chat

pub mod file_share;
pub mod screen_share;
pub mod chat;

// Re-export service types
pub use file_share::MeshFileShare;
pub use screen_share::MeshScreenShare;
pub use chat::MeshChat;
