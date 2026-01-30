# Linux Development Setup

Complete guide for setting up the Robert browser automation project on Linux.

## Quick Setup

Run the automated setup script:

```bash
sudo ./scripts/setup-linux-dev.sh
```

This installs:
- ✅ GTK 3 and WebKit2GTK (for Tauri GUI)
- ✅ Build tools (gcc, make, pkg-config)
- ✅ Google Chrome
- ✅ ChromeDriver  
- ✅ Rust toolchain (if not installed)

**Supported**: Ubuntu, Debian, Linux Mint, Pop!_OS

## Important: Linux Sandbox Issue (Ubuntu 23.10+)

**If you see this error:**
```
FATAL:zygote_host_impl_linux.cc(128)] No usable sandbox!
```

This is because Ubuntu 23.10+ restricts unprivileged user namespaces via AppArmor.

### ✅ Solution: Tests already handle this!

All `headless_integration` tests use `no_sandbox: true` and work correctly:

```rust
ChromeDriver::new(ConnectionMode::Sandboxed {
    chrome_path: None,
    no_sandbox: true,    // ← Handles Ubuntu 23.10+ restrictions
    headless: true,
}).await?
```

The validation tests don't use Chrome at all, so they always work! ✅

## Build and Test

```bash
# Build everything
cargo build

# Run validation tests (always work, no Chrome needed)
cargo test --package robert-webdriver validation

# Run headless tests (work with sandbox restrictions)
cargo test --package robert-webdriver --test headless_integration

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings
```

## Test Status

| Test Suite | Status | Notes |
|------------|--------|-------|
| Validation tests (20 tests) | ✅ Always pass | No Chrome needed |
| Headless integration tests | ✅ Pass | Uses `no_sandbox: true` |
| E2E tests | ⚠️ Sandbox issue | Ubuntu 23.10+ restriction |

## Verification

```bash
# Check installations
google-chrome --version
pkg-config --modversion gtk+-3.0
cargo --version

# Test validation (fastest, no Chrome)
cargo test --package robert-webdriver validation

# Should see: test result: ok. 20 passed
```

## CI/CD

GitHub Actions automatically handles all dependencies and sandbox issues.
See: `.github/workflows/ci.yml`

---

For manual installation or troubleshooting, see the setup script: `scripts/setup-linux-dev.sh`
