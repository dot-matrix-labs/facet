# Robert CDP System - Build Instructions

## Quick Start (No System Dependencies Required)

The CDP system (Chrome DevTools Protocol automation) **does not require Pango or any GTK dependencies**. Only the optional Tauri desktop app requires those.

### Build CDP Tools (No Pango Needed)

```bash
# Build everything (default)
cargo build

# This builds:
# - robert-webdriver (library)
# - robert-cli (robert and robert-generate binaries)
```

### Run Tests

```bash
# Unit tests
cargo test --lib

# Integration tests (headless)
cargo test --test headless_integration -- --test-threads=1

# All tests
cargo test
```

### Run CDP Tools

```bash
# Generate a CDP script with Claude
cargo run --bin robert-generate -- "Take a screenshot of example.com" -o script.json

# Execute the script (if you also want to run it)
cargo run --bin robert-generate -- "Screenshot google.com" --execute --headless
```

---

## Understanding the Workspace

### Default Build (No System Dependencies)

By default, `cargo build` builds:
- ✅ `robert-webdriver` - CDP library
- ✅ `robert-cli` - CLI tools (`robert` and `robert-generate`)
- ❌ `robert-app` - **Excluded** (requires GTK/Pango)

**No system libraries required!** Just Rust and Chrome/Chromium.

### Optional: Build Tauri Desktop App

The Tauri desktop UI is **optional** and requires system dependencies.

**System Dependencies (Linux):**
```bash
# Ubuntu/Debian
sudo apt-get install \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    pango1.0-dev \
    libsoup-3.0-dev

# Fedora
sudo dnf install \
    webkit2gtk4.1-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    pango-devel \
    libsoup3-devel

# Arch
sudo pacman -S \
    webkit2gtk-4.1 \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    pango \
    libsoup3
```

**Then build:**
```bash
# Build the Tauri app specifically
cargo build -p robert-app

# Or include it in workspace build
cargo build --workspace
```

---

## What Needs What

### ✅ CDP System (No System Dependencies)

**Packages:**
- `robert-webdriver` - Core CDP functionality
- `robert-cli` - CLI tools

**Runtime Requirements:**
- Chrome/Chromium (auto-downloads on first use)
- Rust toolchain

**Build Command:**
```bash
cargo build
# or
cargo build --bin robert --bin robert-generate
```

**Works On:**
- ✅ Linux (no GUI needed)
- ✅ macOS
- ✅ Windows
- ✅ CI/CD environments
- ✅ Headless servers
- ✅ Docker containers

---

### ⚠️ Tauri Desktop App (Requires System Dependencies)

**Package:**
- `robert-app` - Desktop UI

**System Dependencies:**
- GTK 3
- WebKit2GTK 4.1
- Pango
- Cairo
- GLib
- Soup 3
- And many more...

**Build Command:**
```bash
cargo build -p robert-app
```

**Works On:**
- ⚠️ Linux (with system libraries)
- ⚠️ macOS (with Xcode)
- ⚠️ Windows (with WebView2)

---

## Troubleshooting

### "pango-sys" or GTK errors

**Problem:** You're trying to build the Tauri app without system dependencies.

**Solutions:**

1. **Just use the CDP system** (recommended):
   ```bash
   cargo build  # Only builds CDP tools by default
   ```

2. **Install system dependencies** (if you want the Tauri UI):
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev pango1.0-dev libsoup-3.0-dev

   # Then
   cargo build --workspace
   ```

3. **Explicitly exclude Tauri**:
   ```bash
   cargo build --workspace --exclude robert-app
   ```

### Chrome not found

**Problem:** Chrome/Chromium not installed.

**Solution:** Don't worry! Chrome auto-downloads on first use:
```bash
cargo run --bin robert-generate -- "test" -o test.json
# Chrome will download automatically
```

Or install manually:
```bash
# Ubuntu/Debian
sudo apt install chromium-browser

# Fedora
sudo dnf install chromium

# macOS
brew install --cask google-chrome
```

### Tests failing in CI

**Problem:** Running in CI/CD without display.

**Solution:** Use headless mode (automatic in CI):
```bash
CI=1 cargo test --test headless_integration -- --test-threads=1
```

---

## Development Workflow

### Working on CDP System

```bash
# Build
cargo build

# Test
cargo test --lib
cargo test --test headless_integration

# Run
cargo run --bin robert-generate -- "Your automation" -o script.json
```

**No system dependencies needed!**

### Working on Tauri App

```bash
# Install system deps first (one time)
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev pango1.0-dev libsoup-3.0-dev

# Build
cargo build -p robert-app

# Run Tauri dev
cd crates/robert-app
npm install
npm run tauri dev
```

---

## CI/CD Configuration

### GitHub Actions Example

```yaml
name: Test CDP System

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: cargo build
        # No system dependencies needed!

      - name: Run tests
        run: |
          cargo test --lib
          cargo test --test headless_integration -- --test-threads=1
        env:
          CI: true
```

**No Pango, no GTK, no system dependencies required!**

---

## Summary

| Component | System Deps? | Build Command | Use Case |
|-----------|-------------|---------------|----------|
| CDP Library | ❌ No | `cargo build` | Automation library |
| CDP CLI Tools | ❌ No | `cargo build` | Command-line automation |
| Headless Tests | ❌ No | `cargo test` | CI/CD testing |
| Tauri Desktop App | ✅ Yes (GTK) | `cargo build -p robert-app` | Desktop GUI |

**TL;DR:** The CDP system works without any system dependencies. Pango is only needed for the optional Tauri desktop UI.

---

## Quick Reference

```bash
# ✅ These work without Pango
cargo build                                  # Build CDP tools
cargo test                                   # Run tests
cargo run --bin robert-generate -- "..." -o script.json

# ⚠️ This needs Pango
cargo build -p robert-app                    # Build Tauri app
cargo build --workspace                      # Build everything including Tauri
```

---

**Status:** ✅ CDP system builds and runs without system dependencies
**Default Build:** Excludes Tauri (no Pango needed)
**Optional UI:** Tauri app available with system deps
