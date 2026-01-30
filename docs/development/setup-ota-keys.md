# OTA Update Keys Setup Guide

This guide walks you through setting up the cryptographic signing keys required for OTA updates.

## Prerequisites

- Bun installed
- Access to repository secrets (for CI/CD)
- Access to `lucky-tensor/robert-releases` repository

## Step 1: Generate Signing Keys

From the repository root:

```bash
# Create directory for keys (this is gitignored)
mkdir -p .tauri-keys

# Generate keypair
cd crates/robert-app
bunx tauri signer generate -w ../../.tauri-keys/robert.key
```

When prompted:
1. **Enter a strong password** (save it in your password manager!)
2. **Press Enter** to confirm

This creates:
- `.tauri-keys/robert.key` - Private key (KEEP SECRET!)
- `.tauri-keys/robert.key.pub` - Public key (safe to distribute)

## Step 2: Update tauri.conf.json

1. **Copy the public key content:**

```bash
cat .tauri-keys/robert.key.pub
```

2. **Edit `crates/robert-app/src-tauri/tauri.conf.json`:**

Find the line:
```json
"pubkey": "PLACEHOLDER_PUBLIC_KEY_WILL_BE_REPLACED_AFTER_KEY_GENERATION",
```

Replace with the actual public key content (including the dashes):
```json
"pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDFCNUE3REJBRDM5NzBFMkIKUldRWlhLelVQZjFNMkVLdGZJRld5cHdZdzd2U0FCWEVxa1VCNWNMZnNRN1VoNHY5blJMUmlxL3IK",
```

3. **Save and commit:**

```bash
git add crates/robert-app/src-tauri/tauri.conf.json
git commit -m "chore: add OTA update public key"
```

## Step 3: Configure GitHub Secrets

### Main Repository (lucky-tensor/robert)

Go to repository → Settings → Secrets and variables → Actions → New repository secret

Add these secrets:

#### 1. TAURI_SIGNING_PRIVATE_KEY

```bash
# Copy the ENTIRE content of the private key file
cat .tauri-keys/robert.key
```

- **Name:** `TAURI_SIGNING_PRIVATE_KEY`
- **Secret:** Paste the entire content (including all lines)

#### 2. TAURI_SIGNING_PRIVATE_KEY_PASSWORD

- **Name:** `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- **Secret:** The password you entered when generating keys

#### 3. RELEASES_REPO_TOKEN

Create a GitHub Personal Access Token:

1. Go to GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Name: "Robert Releases Repo Access"
4. Scopes: Check `repo` (full control of private repositories)
5. Click "Generate token"
6. **Copy the token** (you won't see it again!)

Add to repository secrets:
- **Name:** `RELEASES_REPO_TOKEN`
- **Secret:** Paste the token

## Step 4: Verify Setup

### Local Build Test

Test that signing works locally:

```bash
# Set environment variables
export TAURI_SIGNING_PRIVATE_KEY=$(cat .tauri-keys/robert.key)
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="your_password_here"

# Build
cd crates/robert-app
bunx tauri build

# Verify updater artifacts were created
find ../../target -name "*.app.tar.gz"
find ../../target -name "*.app.tar.gz.sig"
```

You should see output like:
```
../../target/aarch64-apple-darwin/release/bundle/macos/Robert.app.tar.gz
../../target/aarch64-apple-darwin/release/bundle/macos/Robert.app.tar.gz.sig
```

### CI/CD Test

1. Create a test tag:
```bash
git tag test-ota-0.1.2
git push origin test-ota-0.1.2
```

2. Check GitHub Actions workflow progress
3. Verify assets appear in `lucky-tensor/robert-releases`

## Step 5: Security Best Practices

### Backup Private Key

1. **Store in password manager:**
   - Key file content
   - Key password
   - Public key (for reference)

2. **Secure local copy:**
   - Ensure `.tauri-keys/` is in `.gitignore` (already done)
   - Set restrictive permissions:
   ```bash
   chmod 600 .tauri-keys/robert.key
   ```

### Access Control

- **Private key:** Only on your machine and CI/CD
- **Password:** Only you and team leads know it
- **GitHub secrets:** Only repository admins can access

### Key Rotation Plan

If you need to rotate keys (compromised, best practice, etc.):

1. Generate new keypair (different filename)
2. Update public key in code
3. Create new version with new key
4. Update GitHub secrets
5. Release new version
6. **Note:** Old versions won't accept updates signed with new key

## Troubleshooting

### "Please enter a password" Error in CI

**Problem:** Interactive prompt in non-interactive environment.

**Solution:** This is expected when running locally. The GitHub Actions workflow uses environment variables, which work non-interactively.

### "Invalid signature" on Update

**Problem:** Signature verification fails during update.

**Solution:**
1. Verify public key in `tauri.conf.json` is correct
2. Ensure you didn't accidentally use a different key pair
3. Rebuild and re-release

### "TAURI_SIGNING_PRIVATE_KEY must be set"

**Problem:** Environment variable not set during build.

**Solution:**
- **Local:** Use `export` command before building
- **CI/CD:** Ensure `TAURI_SIGNING_PRIVATE_KEY` GitHub secret is set (the workflow maps it to `TAURI_SIGNING_PRIVATE_KEY`)

## Files Checklist

After completing setup:

- [ ] `.tauri-keys/robert.key` exists (gitignored)
- [ ] `.tauri-keys/robert.key.pub` exists (gitignored)
- [ ] Public key added to `tauri.conf.json`
- [ ] `TAURI_SIGNING_PRIVATE_KEY` GitHub secret set
- [ ] `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` GitHub secret set
- [ ] `RELEASES_REPO_TOKEN` GitHub secret set
- [ ] Private key backed up securely
- [ ] Test build completes successfully
- [ ] Updater artifacts (`.tar.gz` + `.sig`) generated

## Next Steps

Once keys are set up:

1. Create a release tag
2. GitHub Actions will build and sign automatically
3. Updater manifest will be generated
4. Users will receive OTA updates

See `docs/OTA_UPDATES.md` for full documentation.
