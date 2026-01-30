# Facet App - Tauri Frontend

This is the Tauri desktop application for Facet, built with Svelte 5, TypeScript, and Vite.

## Prerequisites

- [Bun](https://bun.sh) - Fast JavaScript runtime and package manager
- System dependencies for Tauri (see main project README)

## Development

### First-time setup

```bash
# Install dependencies
bun install

# Set DISPLAY for GUI (Linux only)
export DISPLAY=:0
```

### Running the app

```bash
# Start dev server with Tauri window
bun dev

# Start frontend only (no native window)
bun run frontend:dev
```

## Code Quality

### Linting

```bash
# Check for linting errors
bun run lint

# Fix auto-fixable linting errors
bun run lint:fix
```

### Formatting

```bash
# Check formatting
bun run format

# Fix formatting
bun run format:fix
```

### Type Checking

```bash
# Run Svelte type checker
bun run check
```

### Run all checks

```bash
# Run lint, format, and type checks
bun run check:all
```

## Building

```bash
# Production build
bun build

# Debug build
bun run build:debug
```

## CI/CD

The following checks run in CI:

- TypeScript type checking (`bun run check`)
- ESLint linting (`bun run lint`)
- Prettier formatting (`bun run format`)
- Frontend build (`bun run frontend:build`)

Make sure all checks pass before pushing:

```bash
bun run check:all
```

## Project Structure

```
src/
├── components/     # Svelte components
├── lib/           # Shared utilities and modules
├── App.svelte     # Main app component
├── app.css        # Global styles
└── main.ts        # Entry point

src-tauri/         # Rust backend (Tauri)
```

## Configuration Files

- `eslint.config.js` - ESLint configuration (TypeScript + Svelte)
- `.prettierrc.json` - Prettier formatting rules
- `tsconfig.json` - TypeScript compiler options
- `vite.config.js` - Vite bundler configuration
- `svelte.config.js` - Svelte compiler options
