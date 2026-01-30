# Desk Share Net - Testing Guide

## Running Tests

### All Tests
```bash
cargo test
```

### Unit Tests Only
```bash
cargo test --lib
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### End-to-End Tests
```bash
# Note: E2E tests are ignored by default and require network setup
cargo test --test e2e_tests -- --ignored
```

### With Logging
```bash
RUST_LOG=debug cargo test
```

## Test Categories

### Unit Tests
Located in each module's `tests` submodule:
- `src/error.rs` - Error handling and recovery
- `src/services/chat.rs` - Chat messaging
- `src/platform/windows.rs` - Windows screen capture
- `src/platform/macos.rs` - macOS screen capture
- `src/platform/linux.rs` - Linux screen capture

### Integration Tests
Located in `tests/integration_tests.rs`:
- App state initialization
- Device serialization
- Module integration

### End-to-End Tests
Located in `tests/e2e_tests.rs`:
- File transfer workflow
- Screen sharing sessions
- Chat messaging
- NAT traversal
- Error recovery scenarios

## Test Coverage

Run with coverage (requires `cargo-tarpaulin`):
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Manual Testing

### 1. Start Application
```bash
cargo run
```

### 2. Test Device Discovery
- Open application on two devices on same network
- Verify devices appear in discovery list
- Click refresh to update device list

### 3. Test File Transfer
- Select a file
- Choose target device
- Click "Send to Device"
- Verify progress bar updates
- Check file integrity on receiver

### 4. Test Screen Sharing
- Click "Start Sharing"
- Note the session ID
- On another device, click "Join Session"
- Verify screen updates appear
- Click "Stop Sharing" to end

### 5. Test Chat
- Send a message to a specific peer
- Send a broadcast message
- Verify message delivery
- Check message history

## Troubleshooting Tests

### Network Tests Failing
- Ensure devices are on same network
- Check firewall settings
- Verify ports 5353, 8080 are open

### Screen Capture Tests Failing
- Check screen capture permissions
- On macOS: System Preferences > Security & Privacy > Screen Recording
- On Windows: Check app permissions
- On Linux: Verify X11/Wayland access

### WebRTC Tests Failing
- Verify STUN server is accessible
- Check NAT configuration
- Review ICE candidate generation logs

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --verbose
```

## Performance Testing

### Benchmark File Transfer
```bash
# Transfer 100MB file and measure time
time cargo run -- transfer --file large_file.bin --to 192.168.1.100
```

### Benchmark Screen Sharing
```bash
# Measure frame rate and latency
cargo run -- benchmark-screen-share --duration 60
```

## Known Issues

1. **Platform-specific screen capture** - Native APIs not fully implemented, using fallback
2. **WebRTC signaling** - Requires external signaling server for production
3. **NAT traversal** - TURN server needed for symmetric NAT scenarios

## Contributing Tests

When adding new features:
1. Add unit tests in the module
2. Add integration tests if multiple modules interact
3. Add E2E tests for user-facing workflows
4. Update this guide with new test instructions
