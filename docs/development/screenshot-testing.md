# Screenshot Testing & Troubleshooting Guide

## Overview

This document describes the comprehensive test coverage for screenshot functionality and provides guidance for troubleshooting screenshot-related issues.

## Test Coverage

### Unit Tests (`screenshot_test.rs`)

Located in: `/home/jeef/robert/crates/robert-webdriver/tests/screenshot_test.rs`

**Screenshot Method Tests:**
- ✅ `test_screenshot_returns_valid_png_data` - Verifies screenshot data format
- ✅ `test_screenshot_to_file_creates_valid_file` - Verifies file creation
- ✅ `test_screenshot_multiple_times` - Tests repeated screenshot capture
- ✅ `test_screenshot_different_pages` - Verifies different content produces different screenshots

**CDP Command Tests:**
- ✅ `test_cdp_capture_screenshot_command` - Tests `Page.captureScreenshot` CDP command
- ✅ `test_cdp_screenshot_with_different_formats` - Tests PNG and JPEG formats

**Integration Tests (Step Frames):**
- ✅ `test_screenshot_integration_with_step_frame` - Creates browser step frame with screenshot
- ✅ `test_multiple_step_frames_with_screenshots` - Multiple frames in a workflow
- ✅ `test_step_frame_with_cdp_workflow` - Complete CDP workflow with frame capture

**Error Handling Tests:**
- ✅ `test_screenshot_before_navigation` - Screenshots on blank pages
- ✅ `test_screenshot_to_invalid_path` - Graceful failure handling

**Total: 11 unit/integration tests**

### Hang Detection Tests (`screenshot_hang_test.rs`)

Located in: `/home/jeef/robert/crates/robert-webdriver/tests/screenshot_hang_test.rs`

**Timeout & Hang Prevention:**
- ✅ `test_screenshot_with_timeout` - Detects if screenshots hang beyond 10s
- ✅ `test_screenshot_immediately_after_navigation` - Tests potential race conditions
- ✅ `test_screenshot_to_file_with_timeout` - File save operation timeout detection
- ✅ `test_multiple_screenshots_rapid_succession` - Race condition testing
- ✅ `test_screenshot_after_javascript_execution` - DOM manipulation scenarios
- ✅ `test_screenshot_on_slow_loading_page` - Slow page load scenarios

**Diagnostic Tests:**
- ✅ `test_diagnose_screenshot_performance` - Performance benchmarking
- ✅ `test_screenshot_with_explicit_page_ready_check` - ReadyState verification

**Stress Tests:**
- ✅ `test_screenshot_stress_concurrent_operations` - Concurrent operation handling

**Total: 9 hang detection/performance tests**

## Performance Benchmarks

Based on test results (headless mode, local test server):

| Metric | Value | Status |
|--------|-------|--------|
| Average screenshot time | ~76ms | ✅ Excellent |
| Minimum time | ~66ms | ✅ Excellent |
| Maximum time | ~148ms | ✅ Good |
| Rapid succession (5 shots) | ~360ms | ✅ Good |

**Key Findings:**
- No hangs detected in any test scenario
- Screenshots complete reliably within 150ms
- Performance is consistent across multiple iterations
- Concurrent operations don't cause issues

## Troubleshooting Guide

### Issue: "App stops at 'attempting to take screenshot'"

**Potential Causes & Solutions:**

#### 1. **Navigation Not Complete**

**Symptom:** Screenshot attempt immediately after navigation
**Solution:**
```rust
// The navigate() function already waits for load event
driver.navigate(&url).await?;

// Verify page is ready (optional)
let ready_state: serde_json::Value = driver
    .execute_script("document.readyState")
    .await?;
assert_eq!(ready_state, "complete");

// Now take screenshot
driver.screenshot().await?;
```

**Code Location:** `/home/jeef/robert/crates/robert-app/src-tauri/src/commands/agent.rs:279`

#### 2. **Browser Not Launched**

**Symptom:** Screenshot called before browser initialization
**Check:**
```rust
let driver_lock = state.driver.lock().await;
if driver_lock.is_none() {
    return Err("Browser not launched".to_string());
}
```

**Code Location:** `/home/jeef/robert/crates/robert-app/src-tauri/src/commands/mod.rs:162-164`

#### 3. **File System Issues**

**Symptom:** Cannot write to screenshot path
**Solution:**
```rust
// Ensure directory exists
let temp_dir = std::env::temp_dir().join("robert-chat");
tokio::fs::create_dir_all(&temp_dir).await?;

// Use unique filename
let timestamp = chrono::Utc::now().timestamp();
let screenshot_path = temp_dir.join(format!("screenshot-{}.png", timestamp));
```

**Code Location:** `/home/jeef/robert/crates/robert-app/src-tauri/src/commands/agent.rs:274-277`

#### 4. **Add Timeout Protection**

**Recommended:** Wrap screenshot operations in timeout
```rust
use tokio::time::{timeout, Duration};

// Add 10 second timeout
let result = timeout(
    Duration::from_secs(10),
    driver.screenshot_to_file(&path)
).await;

match result {
    Ok(Ok(())) => { /* Success */ },
    Ok(Err(e)) => { /* Screenshot failed */ },
    Err(_) => { /* Timeout - potential hang detected */ }
}
```

### Issue: Screenshots Are Slow

**Expected Performance:**
- Single screenshot: < 150ms
- File save: < 100ms

**If slower, check:**
1. **Headless mode** - Headed browsers are slower
2. **Page complexity** - Large DOMs take longer
3. **System resources** - CPU/memory constraints

**Diagnostic Test:**
```bash
cargo test test_diagnose_screenshot_performance -- --nocapture
```

### Issue: Screenshots Missing from Step Frames

**Step Frame Format:**
```json
{
  "frame_id": 1,
  "timestamp": "2025-10-11T17:52:13.167180695+00:00",
  "elapsed_ms": 755,
  "screenshot": {
    "path": "/tmp/frame-1.png",
    "format": "png",
    "size_bytes": 24993
  },
  "dom": {
    "url": "http://example.com",
    "title": "Example Page"
  },
  "action": {
    "description": "Navigate to page"
  }
}
```

**Implementation Example:**
```rust
// Take screenshot
let screenshot_path = temp_dir.join("frame-screenshot.png");
driver.screenshot_to_file(&screenshot_path).await?;

// Get metadata
let metadata = tokio::fs::metadata(&screenshot_path).await?;

// Create frame
let frame = BrowserStepFrame {
    frame_id: 1,
    timestamp: chrono::Utc::now().to_rfc3339(),
    elapsed_ms: start_time.elapsed().as_millis() as u64,
    screenshot: ScreenshotInfo {
        path: screenshot_path.to_string_lossy().to_string(),
        format: "png".to_string(),
        size_bytes: metadata.len() as usize,
    },
    dom: DomInfo {
        url: driver.current_url().await?,
        title: driver.title().await?,
    },
    action: ActionInfo {
        description: "Action description".to_string(),
    },
};
```

**See Test:** `test_screenshot_integration_with_step_frame` in `screenshot_test.rs:303`

## Running Tests

### Run All Screenshot Tests
```bash
# Unit and integration tests
cargo test --test screenshot_test -- --test-threads=1 --nocapture

# Hang detection tests
cargo test --test screenshot_hang_test -- --test-threads=1 --nocapture

# All tests together
cargo test screenshot -- --test-threads=1
```

### Run Specific Tests
```bash
# Performance diagnostic
cargo test test_diagnose_screenshot_performance -- --nocapture

# Hang detection
cargo test test_screenshot_with_timeout -- --nocapture

# Step frame integration
cargo test test_screenshot_integration_with_step_frame -- --nocapture
```

## Implementation References

### Core Screenshot Methods

**Location:** `/home/jeef/robert/crates/robert-webdriver/src/browser/chrome.rs`

```rust
// Line 433-443: Take screenshot and return bytes
pub async fn screenshot(&self) -> Result<Vec<u8>>

// Line 446-454: Take screenshot and save to file
pub async fn screenshot_to_file(&self, path: &Path) -> Result<()>
```

### CDP Screenshot Execution

**Location:** `/home/jeef/robert/crates/robert-webdriver/src/cdp/executor.rs`

```rust
// Line 130-162: Execute Page.captureScreenshot command
async fn execute_page_capture_screenshot(
    &self,
    cmd: &CdpCommand,
) -> Result<(Value, Option<String>)>
```

### Command Handlers

**Location:** `/home/jeef/robert/crates/robert-app/src-tauri/src/commands/mod.rs`

```rust
// Line 154-182: take_screenshot command
pub async fn take_screenshot(
    app: AppHandle,
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String>

// Line 280-372: ask_claude_about_page (includes screenshot)
pub async fn ask_claude_about_page(
    app: AppHandle,
    state: State<'_, AppState>,
    prompt: String,
    model: Option<String>,
) -> Result<ClaudeResponse, String>
```

**Location:** `/home/jeef/robert/crates/robert-app/src-tauri/src/commands/agent.rs`

```rust
// Line 267-287: capture_screenshot_if_available helper
async fn capture_screenshot_if_available(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Option<PathBuf>
```

## Best Practices

### 1. Always Check Browser State
```rust
if driver_lock.is_none() {
    return Err("Browser not launched");
}
```

### 2. Ensure Directory Exists
```rust
tokio::fs::create_dir_all(&screenshot_dir).await?;
```

### 3. Use Unique Filenames
```rust
let timestamp = chrono::Utc::now().timestamp();
let path = dir.join(format!("screenshot-{}.png", timestamp));
```

### 4. Add Timeout Protection
```rust
timeout(Duration::from_secs(10), driver.screenshot()).await?
```

### 5. Verify Screenshot Data
```rust
let screenshot = driver.screenshot().await?;
assert!(screenshot.len() > 1000, "Screenshot should have data");
assert_eq!(screenshot[0..4], [0x89, 0x50, 0x4E, 0x47], "Should be PNG");
```

### 6. Clean Up Temporary Files
```rust
// After use
tokio::fs::remove_file(&screenshot_path).await?;
```

## Debug Logging

Enable verbose logging to diagnose screenshot issues:

```bash
RUST_LOG=debug cargo run
```

**Key Log Messages:**
- `🌐 Starting navigation to: {url}`
- `✓ Navigation completed successfully`
- `emit_info: "Taking screenshot..."`
- `emit_success: "Screenshot saved to: {path}"`

## Related Documentation

- [step_frame_schema.md](../agent-formats/specs/step_frame_schema.md) - Step frame format specification
- [testing.md](testing.md) - Overall testing strategy
- [TEST_ORGANIZATION.md](TEST_ORGANIZATION.md) - Test organization guide

## Summary

**Total Test Coverage: 20 tests**
- ✅ 11 unit/integration tests
- ✅ 9 hang detection tests
- ✅ 100% pass rate
- ✅ No hangs detected
- ✅ Performance within acceptable ranges

**Next Steps for Users:**

If experiencing screenshot hangs:
1. Run diagnostic tests to reproduce the issue
2. Check browser launch status
3. Verify navigation completion
4. Add timeout protection
5. Check file system permissions
6. Review logs for error messages

**For Developers:**

The test suite is comprehensive and ready for CI/CD integration. All tests pass reliably in headless mode with good performance characteristics.
