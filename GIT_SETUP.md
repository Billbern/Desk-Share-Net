# Git Setup Guide

## Initialize Git Repository

```powershell
# Navigate to project directory
cd c:\Users\HP\Documents\Dev\Desk_Share_Net

# Initialize git repository
git init

# Add all files
git add .

# Create initial commit
git commit -m "Initial commit: Desk_Share_Net P2P desktop sharing app"
```

## What's Ignored

The `.gitignore` file excludes:

### Build Artifacts
- `target/` - Rust build output
- `dist/`, `build/`, `out/` - Distribution builds
- `src-tauri/target/` - Tauri build output
- `*.exe`, `*.dll`, `*.so` - Compiled binaries

### Dependencies
- `node_modules/` - Node.js packages
- `Cargo.lock` - Rust dependency lock (included in libraries, excluded in binaries)
- `package-lock.json`, `yarn.lock` - Node lock files

### IDE/Editor Files
- `.vscode/` - VS Code settings
- `.idea/` - JetBrains IDEs
- `*.swp`, `*.swo` - Vim swap files
- `.DS_Store` - macOS metadata

### Temporary Files
- `*.log` - Log files
- `*.tmp` - Temporary files
- `*.bak`, `*.backup` - Backup files

### OS-Specific
- `Thumbs.db` - Windows thumbnails
- `.DS_Store` - macOS metadata
- `.Trash-*` - Linux trash

## What's Tracked

These important files ARE tracked:
- ✅ All source code (`src/`, `src-tauri/src/`)
- ✅ Configuration files (`Cargo.toml`, `tauri.conf.json`)
- ✅ Documentation (`README.md`, `*.md`)
- ✅ Tests (`tests/`)
- ✅ UI files (`index.html`, `*.css`, `*.js`)
- ✅ Package manifests (`package.json`)

## Create GitHub Repository

### Option 1: Using GitHub CLI
```powershell
# Install GitHub CLI
winget install GitHub.cli

# Authenticate
gh auth login

# Create repository
gh repo create desk-share-net --public --source=. --remote=origin

# Push code
git push -u origin main
```

### Option 2: Using Web Interface
1. Go to https://github.com/new
2. Create repository named `desk-share-net`
3. Don't initialize with README (we have one)
4. Run:
```powershell
git remote add origin https://github.com/YOUR_USERNAME/desk-share-net.git
git branch -M main
git push -u origin main
```

## Useful Git Commands

```powershell
# Check status
git status

# View changes
git diff

# Add specific files
git add src/main.rs

# Commit changes
git commit -m "Add feature X"

# Push to GitHub
git push

# Pull latest changes
git pull

# Create new branch
git checkout -b feature/new-feature

# View commit history
git log --oneline --graph
```

## Recommended Branches

```
main          - Stable, production-ready code
develop       - Integration branch for features
feature/*     - New features
bugfix/*      - Bug fixes
hotfix/*      - Urgent production fixes
```

## Pre-commit Hooks (Optional)

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Format code before commit
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

Make it executable:
```powershell
chmod +x .git/hooks/pre-commit  # On WSL/Linux/macOS
```

## GitHub Actions CI/CD (Optional)

Create `.github/workflows/ci.yml`:

```yaml
name: CI

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
      - run: cargo test --all
      - run: cargo build --release
```

## .gitattributes (Optional)

Create `.gitattributes` for consistent line endings:

```
* text=auto
*.rs text eol=lf
*.toml text eol=lf
*.md text eol=lf
*.json text eol=lf
*.html text eol=lf
*.css text eol=lf
*.js text eol=lf
```

---

*Git setup complete!* 🎉
