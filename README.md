# README.md

# 🖥️ Desk_Share_Net

A peer-to-peer desktop sharing application built with Rust, Tauri, and WebRTC.

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-1.5-blue.svg)](https://tauri.app/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## ✨ Features

- 🖥️ **Screen Sharing**: Real-time screen sharing with configurable quality
- 📁 **File Transfer**: Chunked file transfer with integrity verification
- 💬 **Chat**: Direct messaging, broadcast, and group chat rooms
- 🌐 **P2P Mesh Network**: Decentralized architecture using libp2p
- 🔒 **NAT Traversal**: STUN/TURN support for connectivity
- 🎯 **Cross-Platform**: Windows, macOS, and Linux support

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Node.js** (16+): For Tauri development
- **C++ Build Tools**:
  - **Windows**: Visual Studio Build Tools with C++
  - **macOS**: Xcode Command Line Tools
  - **Linux**: build-essential, pkg-config

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/desk-share-net.git
cd desk-share-net

# Install dependencies
cargo build

# Run in development mode
cargo tauri dev
```

### Windows Build Tools

```powershell
# Quick install using winget
winget install Microsoft.VisualStudio.2022.BuildTools

# Or download manually from:
# https://visualstudio.microsoft.com/downloads/
```

## 📖 Usage

### Starting the Application

```bash
# Development mode
cargo tauri dev

# Production build
cargo tauri build
```

### Basic Operations

#### Screen Sharing
```rust
// Start sharing
let session_id = screen_share.start_sharing(
    peer_id,
    frame_rate: 30,
    resolution: (1920, 1080)
).await?;

// Join session
screen_share.join_session(&session_id, peer_id).await?;
```

#### File Transfer
```rust
// Share a file
let file_hash = file_transfer.share_file(
    Path::new("/path/to/file"),
    peer_id
).await?;

// Download file
file_transfer.download_file(&file_hash, output_path).await?;
```

#### Chat
```rust
// Send message
let message = chat.send_message(
    "Hello!".to_string(),
    Some(peer_id)
).await?;

// Create chat room
let room = chat.create_room(
    "room1".to_string(),
    vec![peer1, peer2]
).await?;
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│           Tauri Frontend (HTML/JS)      │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│         Tauri Backend (Rust)            │
│  ┌──────────┬──────────┬──────────┐    │
│  │  Screen  │   File   │   Chat   │    │
│  │  Share   │ Transfer │ Service  │    │
│  └────┬─────┴────┬─────┴────┬─────┘    │
└───────┼──────────┼──────────┼──────────┘
        │          │          │
┌───────▼──────────▼──────────▼──────────┐
│         P2P Network Layer (libp2p)      │
│  ┌──────────┬──────────┬──────────┐    │
│  │ Discovery│ Signaling│ Transport│    │
│  └──────────┴──────────┴──────────┘    │
└─────────────────────────────────────────┘
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run E2E tests (requires network setup)
cargo test --test e2e_tests -- --ignored

# With logging
RUST_LOG=debug cargo test
```

## 📚 Documentation

- [Testing Guide](TESTING.md) - How to run and write tests
- [Build Status](BUILD_STATUS.md) - Build requirements and troubleshooting
- [Quick Build Guide](QUICK_BUILD_GUIDE.md) - Fast setup instructions
- [Integration Fixes](INTEGRATION_FIXES.md) - Module integration details
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md) - Feature overview

## 🛠️ Development

### Project Structure

```
desk-share-net/
├── src/
│   ├── main.rs              # Application entry point
│   ├── error.rs             # Error handling
│   ├── network/             # Network layer
│   ├── p2p/                 # P2P networking
│   ├── services/            # High-level services
│   ├── platform/            # Platform-specific code
│   └── ui/                  # UI layer
├── src-tauri/               # Tauri backend
├── tests/                   # Test suites
└── docs/                    # Documentation
```

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check without building
cargo check
```

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📋 Roadmap

### v0.2.0 (Next Release)
- [ ] Complete native screen capture implementations
- [ ] Deploy dedicated signaling server
- [ ] Add peer authentication
- [ ] Performance benchmarking

### v0.3.0
- [ ] Multi-monitor support
- [ ] Audio streaming
- [ ] Remote control capabilities
- [ ] Mobile app support

### v1.0.0
- [ ] Production-ready security
- [ ] Enterprise features
- [ ] Cloud integration
- [ ] Comprehensive documentation

## ⚠️ Known Issues

- Native screen capture APIs are placeholder implementations
- Requires Visual Studio Build Tools on Windows
- E2E tests need network configuration
- TURN server required for symmetric NAT

See [BUILD_STATUS.md](BUILD_STATUS.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop app framework
- [libp2p](https://libp2p.io/) - P2P networking library
- [WebRTC](https://webrtc.org/) - Real-time communication
- [Tokio](https://tokio.rs/) - Async runtime

## 📞 Support

- 📧 Email: support@example.com
- 💬 Discord: [Join our server](https://discord.gg/example)
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/desk-share-net/issues)

---

**Built with ❤️ using Rust and Tauri**
"# Desk-Share-Net" 
