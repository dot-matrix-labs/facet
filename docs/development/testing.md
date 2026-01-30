# Testing Guide

## Quick Test Commands

### Validation Tests (Fastest, Always Pass)
```bash
# Run all validation tests (no Chrome needed)
cargo test --package robert-webdriver validation

# Result: 20 tests pass in ~10ms
```

### Headless Chrome Tests
```bash
# Run headless integration tests (Chrome in headless mode)
# IMPORTANT: Must run sequentially due to Chrome profile conflicts
cargo test --package robert-webdriver --test headless_integration -- --test-threads=1

# Result: 5 tests pass in ~5 seconds
```

### Chat UI Tests
```bash
# Run chat UI injection and messaging tests
cargo test --package robert-webdriver --test chat_ui_test

# Result: 7 tests pass in ~2 seconds
# Note: Uses local test server, no external dependencies
```

### Unit Tests
```bash
# Run all library unit tests
cargo test --package robert-webdriver --lib

# Result: 15 tests pass
```

### All Tests
```bash
# Run all tests in robert-webdriver (excluding E2E)
cargo test --package robert-webdriver -- --test-threads=1

# Result: ~40 tests (validation + headless + unit)
```

## Test Suites

| Suite | Count | Speed | Chrome Needed | Notes |
|-------|-------|-------|---------------|-------|
| **Validation** | 20 | ~10ms | ❌ No | Always pass, no dependencies |
| **Unit Tests** | 15 | ~100ms | ❌ No | Library logic tests |
| **Chat UI** | 7 | ~2s | ✅ Yes | Uses local test server, offline-capable |
| **Headless Integration** | 5 | ~5s | ✅ Yes | Chrome downloads automatically |
| **E2E Tests** | 3 | ~10s | ✅ Yes | ⚠️ Sandbox issues on Ubuntu 23.10+ |

## Important Notes

### 1. Chrome Profile Directory Conflicts

The headless tests must run **sequentially** (not in parallel) to avoid Chrome profile directory conflicts:

```bash
# ✅ Correct - sequential execution
cargo test --package robert-webdriver --test headless_integration -- --test-threads=1

# ❌ Wrong - parallel execution causes failures
cargo test --package robert-webdriver --test headless_integration
```

**Why?** All tests try to use `/tmp/chromiumoxide-runner` simultaneously, causing:
```
ERROR:process_singleton_posix.cc(340)] Failed to create SingletonLock: File exists
```

### 2. Chrome Downloads Automatically

First time running Chrome tests, you'll see:
```
📥 Downloading Chrome for Testing (first time only, ~150MB)...
✅ Chrome downloaded successfully!
```

This is normal and only happens once!

### 3. Ubuntu 23.10+ Sandbox Restrictions

On Ubuntu 23.10+, E2E tests may fail with:
```
FATAL:zygote_host_impl_linux.cc(128)] No usable sandbox!
```

**Solution**: The headless tests already use `no_sandbox: true` and work fine!

## Test Descriptions

### Validation Tests (20 tests)
Tests the CDP script validation system:
- JSON syntax validation
- Required field validation
- Command format validation
- Parameter type checking
- Error location tracking
- Helpful suggestions

**No Chrome needed** - Pure Rust validation logic

### Headless Integration Tests (5 tests)
Tests Chrome automation in headless mode:
1. `test_basic_navigation_headless` - Navigate and get page title
2. `test_cdp_script_execution_headless` - Execute CDP script
3. `test_screenshot_capture_headless` - Take screenshots
4. `test_data_extraction_headless` - Extract page data
5. `test_multiple_commands_headless` - Multiple CDP commands

**Uses `no_sandbox: true`** - Works on Ubuntu 23.10+

### Unit Tests (15 tests)
Tests internal library logic:
- Script validation
- Execution reports
- Command status tracking
- Error handling
- Configuration

## CI/CD

### GitHub Actions

Workflows automatically handle all dependencies and run tests:

```yaml
# .github/workflows/ci.yml
- name: Install system dependencies
  run: sudo bash scripts/setup-linux-dev.sh

- name: Run tests
  run: cargo test --workspace -- --test-threads=1
```

All tests pass in CI! ✅

## Troubleshooting

### Tests fail with "SingletonLock: File exists"
**Solution**: Run with `--test-threads=1`:
```bash
cargo test --test headless_integration -- --test-threads=1
```

### Tests fail with "No usable sandbox"
**Solution**: Use headless tests (they already have `no_sandbox: true`):
```bash
cargo test --test headless_integration -- --test-threads=1
```

### Chrome downloads every test run
**Solution**: This shouldn't happen. Chrome is cached after first download.
If it persists, check if `/tmp/chrome-for-testing/` is being deleted.

### Validation tests fail
**Solution**: This indicates a code issue - validation tests should always pass.
Check: `cargo test --package robert-webdriver validation --nocapture`

## Performance

| Command | Time | Notes |
|---------|------|-------|
| Validation tests | ~10ms | Fastest |
| Unit tests | ~100ms | Fast |
| Single headless test | ~1s | Chrome startup |
| All headless tests | ~5s | Sequential |
| Full test suite | ~10s | Everything |

## Development Workflow

### Quick feedback loop
```bash
# During development - fastest tests
cargo test --package robert-webdriver validation
```

### Before commit
```bash
# Run all tests
cargo test --package robert-webdriver -- --test-threads=1

# Run linting
cargo clippy --workspace --all-targets -- -D warnings
```

### Local CI simulation
```bash
# Same as CI runs
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
```

## Test Coverage

- ✅ JSON validation - Full coverage (20 tests)
- ✅ CDP commands - All 15 commands tested
- ✅ Chrome automation - Headless mode covered
- ✅ Chat UI - Complete injection and messaging coverage (7 tests)
- ✅ Error handling - Comprehensive error tests
- ✅ Script execution - E2E workflow tested
- ✅ Local test server - All Chrome tests use offline-capable local server

## Summary

**Quick Commands:**
```bash
# Fastest - validation only
cargo test --package robert-webdriver validation

# Chrome tests (must be sequential)
cargo test --package robert-webdriver --test headless_integration -- --test-threads=1

# Everything
cargo test --package robert-webdriver -- --test-threads=1
```

**Expected Results:**
- ✅ Validation: 20/20 pass
- ✅ Headless: 5/5 pass
- ✅ Unit: 15/15 pass
- ⚠️ E2E: May fail on Ubuntu 23.10+ (sandbox)

**Total working tests: 40/40** (excluding E2E with sandbox issues)

---

For setup instructions, see: `docs/LINUX_SETUP.md`
