# Over-the-Air (OTA) Automatic Updates

This document describes the OTA automatic update system implemented for Robert, the browser automation desktop app.

## Overview

Robert uses Tauri v2's updater plugin to provide secure, cryptographically-signed automatic updates. The system:

- **Checks for updates** automatically on app startup (after 3 second delay)
- **Shows custom UI** for update notifications and progress
- **Downloads and verifies** updates with Ed25519 signatures
- **Relaunches automatically** after successful installation
- **Supports manual checks** via the update button in the header

## Architecture

### Components

1. **Frontend (TypeScript/Svelte)**
   - `src/lib/updater.ts` - Update checking and installation logic
   - `src/components/UpdateModal.svelte` - Custom update UI
   - `src/App.svelte` - Integration with main app

2. **Backend (Rust)**
   - `tauri-plugin-updater` - Core update functionality
   - `tauri-plugin-dialog` - User prompts (not used with custom UI)
   - `tauri-plugin-process` - App relaunch

3. **CI/CD (GitHub Actions)**
   - `.github/workflows/release.yml` - Build and release workflow
   - Generates updater bundles (`.app.tar.gz` + `.sig`)
   - Creates `latest.json` manifest
   - Uploads to `lucky-tensor/robert-releases` repository

## Configuration

### Tauri Configuration

Located in `crates/robert-app/src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/lucky-tensor/robert-releases/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY_HERE",
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```

**Important:** The `pubkey` field must contain your actual Ed25519 public key after generating signing keys.

### Capabilities

Located in `crates/robert-app/src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "updater:default",
    "updater:allow-check",
    "updater:allow-download-and-install",
    "dialog:default",
    "process:default",
    "process:allow-relaunch"
  ]
}
```

## Signing Keys

### Generating Keys

**IMPORTANT:** You must generate signing keys before building releases with OTA updates.

```bash
cd crates/robert-app
bunx tauri signer generate -w ../../.tauri-keys/robert.key
```

This will prompt for a password and generate:
- `../../.tauri-keys/robert.key` - **Private key** (KEEP SECRET!)
- `../../.tauri-keys/robert.key.pub` - **Public key** (safe to distribute)

### Configuring Keys

1. **Add public key to `tauri.conf.json`:**

```bash
# Copy the entire content of the public key file
cat .tauri-keys/robert.key.pub
```

Paste the content into the `pubkey` field in `tauri.conf.json`.

2. **Add GitHub Secrets:**

Go to your repository settings → Secrets and variables → Actions:

- **TAURI_SIGNING_PRIVATE_KEY**: Paste the entire content of `.tauri-keys/robert.key`
- **TAURI_SIGNING_PRIVATE_KEY_PASSWORD**: Your key password
- **RELEASES_REPO_TOKEN**: GitHub Personal Access Token with `repo` scope for `lucky-tensor/robert-releases`

**Note:** The workflow uses `TAURI_SIGNING_PRIVATE_KEY` but maps it to the `TAURI_SIGNING_PRIVATE_KEY` environment variable that Tauri expects.

### Local Development

For local builds with updater artifacts:

```bash
# Set environment variables
export TAURI_SIGNING_PRIVATE_KEY=$(cat .tauri-keys/robert.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your_password"

# Build
cd crates/robert-app
bunx tauri build
```

## Update Manifest Format

The `latest.json` file hosted on `robert-releases` follows this format:

```json
{
  "version": "0.1.1",
  "notes": "Release 0.1.1 - See GitHub release for full details",
  "pub_date": "2025-10-11T14:30:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/lucky-tensor/robert-releases/releases/download/0.1.1/Robert_0.1.1_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "url": "https://github.com/lucky-tensor/robert-releases/releases/download/0.1.1/Robert_0.1.1_x64.app.tar.gz"
    }
  }
}
```

**Important:** The `signature` field contains the actual content of the `.sig` file, NOT a URL or path.

## Release Process

### 1. Prepare Release

Update version numbers in:
- `crates/robert-app/src-tauri/Cargo.toml`
- `crates/robert-app/src-tauri/tauri.conf.json`
- `crates/robert-app/package.json`

### 2. Create Release Tag

```bash
git tag 0.1.1
git push origin 0.1.1
```

Or use GitHub's workflow dispatch.

### 3. Automated Build Process

The workflow automatically:

1. **Creates GitHub Release** (draft)
2. **Builds for macOS** (Intel + Apple Silicon)
   - Generates DMG installers
   - Generates updater bundles (`.app.tar.gz`)
   - Signs bundles (`.app.tar.gz.sig`)
3. **Uploads to `robert-releases`:**
   - DMG files for user downloads
   - Updater bundles for OTA updates
4. **Generates `latest.json`** manifest
5. **Uploads manifest** to `robert-releases`
6. **Publishes release** (removes draft status)

### 4. Verify Release

Check `lucky-tensor/robert-releases` for:
- ✅ DMG files uploaded
- ✅ Updater bundles (`.app.tar.gz`) uploaded
- ✅ `latest.json` present and valid

## User Experience

### Automatic Check on Startup

1. User opens Robert app
2. After 3 seconds, app checks for updates silently
3. If update available → modal appears
4. If no update → app continues normally

### Update Modal

The custom modal shows:
- Current version
- New version available
- Release notes
- Download progress with percentage
- Auto-relaunch notification

### Manual Check

Users can click the 🔄 button in the header to manually check for updates anytime.

## Testing

### Testing Update Flow

To test the update flow without publishing:

1. Build a release with signing:
```bash
export TAURI_SIGNING_PRIVATE_KEY=$(cat .tauri-keys/robert.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="password"
cd crates/robert-app
bunx tauri build
```

2. Manually create a `latest.json` file
3. Host it on a local server or GitHub
4. Update `tauri.conf.json` endpoint to point to your test URL
5. Install the built app and trigger update check

### Verifying Artifacts

After building, check for updater artifacts:

```bash
find target -name "*.app.tar.gz"
find target -name "*.app.tar.gz.sig"
```

## Troubleshooting

### No Updater Artifacts Generated

**Problem:** `.app.tar.gz` files not created during build.

**Solution:**
- Ensure `createUpdaterArtifacts: true` in `tauri.conf.json`
- Ensure signing environment variables are set
- Check build logs for signing errors

### Signature Verification Failed

**Problem:** Update downloads but fails to install with signature error.

**Solution:**
- Verify public key in `tauri.conf.json` matches private key used for build
- Ensure signature in `latest.json` is complete content from `.sig` file
- Rebuild and re-sign if keys don't match

### Update Not Detected

**Problem:** App says "no updates available" when update exists.

**Solution:**
- Verify version in `latest.json` is greater than current app version
- Check endpoint URL is accessible
- Validate JSON format
- Check browser console for HTTP errors

### Environment Variables Not Working

**Problem:** Build fails with "set TAURI_SIGNING_PRIVATE_KEY" error.

**Solution:**
- Don't use `.env` files (they don't work for these variables)
- Use `export` on Linux/macOS or `$env:` on Windows
- For GitHub Actions, ensure `TAURI_SIGNING_PRIVATE_KEY` secret is set (the workflow maps it to `TAURI_SIGNING_PRIVATE_KEY`)

## Security Considerations

1. **Private Key Protection:**
   - Never commit `.tauri-keys/robert.key` to git
   - Store securely in password manager
   - Limit access to CI/CD secrets

2. **Signature Verification:**
   - Every update is cryptographically verified
   - Invalid signatures abort the update
   - Cannot be disabled (security feature)

3. **HTTPS Only:**
   - All update endpoints must use HTTPS
   - Prevents man-in-the-middle attacks

4. **Key Rotation:**
   - To rotate keys, generate new keypair
   - Update public key in app
   - Release new version
   - Old versions won't accept updates signed with new key

## Update Channels (Future)

Currently, all users receive the same updates. Future improvements could include:

- **Stable channel:** Main releases only
- **Beta channel:** Pre-release testing
- **Canary channel:** Nightly builds

This would require:
- Separate manifest URLs per channel
- User setting to choose channel
- Dynamic endpoint configuration

## References

- [Tauri v2 Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [Tauri Signing Documentation](https://v2.tauri.app/distribute/sign/)
- [GitHub Actions for Tauri](https://v2.tauri.app/distribute/pipelines/github/)
