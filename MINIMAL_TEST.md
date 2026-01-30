# Minimal Test - No Build Tools Required

This creates a lightweight version to test core logic without native dependencies.

## Create Test Project

```powershell
# Create new minimal project
cargo new --lib desk_share_test
cd desk_share_test
```

## Copy Core Logic

```powershell
# Copy error handling (no native deps)
Copy-Item ..\Desk_Share_Net\src\error.rs .\src\

# Update Cargo.toml
@"
[package]
name = "desk_share_test"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.0", features = ["full"] }
"@ | Out-File -FilePath Cargo.toml -Encoding utf8
```

## Test It

```powershell
# This should work without build tools
cargo test
```

## What This Tests

- ✅ Error handling logic
- ✅ Recovery strategies
- ✅ Retry mechanisms
- ✅ Type system correctness

## What It Doesn't Test

- ❌ Full application
- ❌ UI integration
- ❌ Platform-specific features
- ❌ Network operations

---

This is useful for:
- Testing core logic
- Learning Rust patterns
- Verifying algorithms
- Quick iterations

But for the full app, you need the complete build.
