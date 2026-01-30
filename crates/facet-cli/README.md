# Facet CLI - Browser Automation Tool

A command-line interface for browser automation using Chrome DevTools Protocol (CDP).

## Installation

**No installation required!** Facet automatically downloads Chrome for Testing on first run (~150MB).

Optionally, you can use your system Chrome installation.

### Optional: Install Chrome/Chromium

**Ubuntu/Debian:**
```bash
sudo apt install chromium-browser
```

**Fedora:**
```bash
sudo dnf install chromium
```

**macOS:**
```bash
brew install --cask google-chrome
```

**Or download from:** https://www.google.com/chrome/

## Usage

### Basic Usage (Auto-download Chrome)

```bash
# Navigate to a URL and print HTML (URL protocol optional)
cargo run --bin facet -- example.com
cargo run --bin facet -- https://example.com

# Print visible text only
cargo run --bin facet -- example.com --format text

# Extract specific element
cargo run --bin facet -- example.com --selector "h1"

# Run in headless mode (no visible window)
cargo run --bin facet -- example.com --headless
```

### Custom Chrome Path

If Chrome isn't auto-detected, specify the path:

```bash
cargo run --bin facet -- https://example.com --chrome-path /usr/bin/chromium
```

### Advanced Mode (Connect to Running Chrome)

Connect to an existing Chrome instance with your profile:

```bash
# First, start Chrome with remote debugging
google-chrome --remote-debugging-port=9222

# Then connect to it
cargo run --bin facet -- https://example.com --debug-port 9222
```

## Options

```
facet <URL> [OPTIONS]

Arguments:
  <URL>  URL to navigate to (https:// prefix optional)

Options:
  --debug-port <PORT>       Connect to existing Chrome debug port (advanced mode)
  --chrome-path <PATH>      Path to Chrome/Chromium executable
  --headless                Run Chrome in headless mode (no visible window)
  --no-sandbox              Disable Chrome sandbox (Linux AppArmor workaround)
  -f, --format <FORMAT>     Output format: html or text [default: html]
  -s, --selector <SELECTOR> CSS selector for specific element
  -h, --help                Print help
  -V, --version             Print version
```

## CI/CD Support

Facet automatically detects CI environments and runs in headless mode with `--no-sandbox`:
- GitHub Actions (`GITHUB_ACTIONS`)
- GitLab CI (`GITLAB_CI`)
- Jenkins (`JENKINS_HOME`)
- CircleCI (`CIRCLECI`)
- Generic CI (`CI`)

## Modes

### 1. Sandboxed Mode (Default)
- Auto-downloads Chrome for Testing (first run only, cached at `~/.cache/facet/chrome`)
- Isolated session (no cookies/history)
- Visible browser window (unless `--headless` specified)
- Best for: Testing, automation, scripts

### 2. Advanced Mode (--debug-port)
- Connects to your running Chrome
- Uses your real profile (logged-in accounts, cookies, etc.)
- Requires Chrome started with `--remote-debugging-port`
- Best for: Authenticated workflows, personal automation

### 3. Custom Path (--chrome-path)
- Specify exact Chrome/Chromium binary
- Useful when multiple versions installed
- Useful when Chrome not in standard location

### 4. Headless Mode (--headless)
- No visible browser window
- Faster execution
- Required for CI/CD environments (auto-enabled)

### 5. No Sandbox Mode (--no-sandbox)
- Disables Chrome sandbox (reduces security)
- Required on Ubuntu 23.10+ and systems with AppArmor restrictions
- Auto-enabled in CI/CD environments

## Examples

```bash
# Basic navigation (protocol optional)
facet github.com
facet https://github.com

# Get page title area
facet github.com --selector "title"

# Headless mode
facet github.com --headless

# Get visible text only
facet github.com --format text

# Use specific Chrome
facet github.com --chrome-path /usr/bin/chromium

# Linux with AppArmor restrictions
facet github.com --no-sandbox

# Connect to running Chrome (macOS)
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222 &
facet github.com --debug-port 9222

# CI/CD usage (auto-detects CI environment)
CI=true facet example.com  # Runs headless with --no-sandbox
```

## Troubleshooting

**Error: "Chrome/Chromium not found"**

This should rarely happen as Chrome auto-downloads. If it does:
1. Check cache: `ls ~/.cache/facet/chrome`
2. Manually install Chrome (see Installation above)
3. Specify path: `--chrome-path /path/to/chrome`
4. Use advanced mode: `--debug-port 9222`

**Error: "Failed to connect to Chrome on port 9222"**

Solution:
1. Make sure Chrome is running with: `chrome --remote-debugging-port=9222`
2. Check no other process is using port 9222
3. Try a different port number

**Error: "Browser process exited" or "No usable sandbox"**

Linux (Ubuntu 23.10+) AppArmor issue. Solutions:
1. Use `--no-sandbox` flag: `facet example.com --no-sandbox`
2. Or set CI env var: `CI=true facet example.com`

**Chrome opens and closes immediately**

Make sure to include URL protocol or let Facet add it automatically:
- ✅ `facet example.com` (auto-adds https://)
- ✅ `facet https://example.com`
- ❌ `facet example.com` (if protocol detection fails)

## Technology

- **Library:** spider_chrome (maintained chromiumoxide fork)
- **Protocol:** Chrome DevTools Protocol (CDP)
- **Runtime:** Tokio async
- **Language:** Rust

## License

MIT OR Apache-2.0
