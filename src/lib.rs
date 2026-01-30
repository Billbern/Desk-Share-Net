// Desk Share Net - P2P Screen Sharing and File Transfer Library
//
// This library provides the core functionality for peer-to-peer networking,
// file sharing, screen sharing, and chat services.

pub mod p2p;
pub mod services;
pub mod ui;
pub mod error;
pub mod app;

// Re-export commonly used types
pub use app::{AppState, Device};
pub use error::DeskShareError;

// Re-export network types for convenience
pub use p2p::{NetworkDiscovery, P2PNetwork};
pub use services::{FileTransfer, ScreenShare, ChatService};
