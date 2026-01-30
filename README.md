# Robert: AI-Powered WebDriver

Robert is a webdriver library for leveraging AI to drive browsers, headlessly or visually. It provides the infrastructure to hook AI agents into browser automation workflows.

## Components

### robert-webdriver

The CLI tool and Rust library for connecting AI agents to browser drivers.

- **CLI**: Command-line interface for browser automation tasks
- **Library**: Rust crate for integrating browser control into your AI agent workflows
- Supports headless and visual browser modes
- Built for reliability and performance with Rust

### robert-browser

End-user demonstration application with a GUI. A Tauri-based desktop app showcasing Robert's capabilities in an accessible interface.

## Quick Start

### Prerequisites

- **Rust** 1.75+ (for building from source)
- **Node.js** 18+ (for robert-browser frontend)
- **Chrome/Chromium** (auto-downloads on first use)

### Build and Run

```bash
# Clone repository
git clone https://github.com/dot-matrix-labs/robert-browser.git
cd robert

# Build all components
cargo build --workspace

# Run the CLI
cargo run --bin robert

# Run the browser GUI
cd crates/robert-app
npm install
npm run tauri dev
```

### Using the Library

```rust
use robert_server::prelude::*;

// Connect your AI agent to browser automation
// ... usage examples to come
```

## Project Structure

```
robert/
├── crates/
│   ├── robert-server/    # WebDriver server and library
│   ├── robert-cli/       # CLI tool (robert-webdriver)
│   ├── robert-app/       # Desktop GUI (robert-browser)
│   └── types/            # Shared types
└── README.md
```

## Development

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p robert-server
```

### Code Quality

```bash
# Format code
cargo fmt --workspace

# Lint
cargo clippy --workspace

# Type check
cargo check --workspace
```
