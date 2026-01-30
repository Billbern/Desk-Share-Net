// P2P networking module
// Handles peer discovery, signaling, transport, and network management

pub mod network;
pub mod discovery;
pub mod signalling; // Note: using British spelling as per file name
pub mod transport;

// Common type definitions
pub type PeerId = String;

// Re-export commonly used types
pub use network::P2PNetwork;
pub use discovery::NetworkDiscovery;
pub use signalling::SignalingServer;
pub use transport::P2PTransport;
