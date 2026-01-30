# Robert WebDriver Test Suite Organization

## Test Categories

### 🔧 Meta/Infrastructure Tests (`meta_infrastructure_test.rs`)
**Purpose**: Verify testing infrastructure itself
**Chrome Required**: ❌ No
**Tests**: 4

- `meta_test_server_starts` - Test server starts on random port
- `meta_test_server_serves_html` - Test server serves correct HTML
- `meta_test_multiple_servers_different_ports` - Isolation with different ports
- `meta_test_server_wait_ready` - Server readiness detection

```bash
cargo test --package robert-webdriver --test meta_infrastructure_test
```

---

### ✅ Validation Tests (`validation_test.rs`)
**Purpose**: CDP JSON schema validation
**Chrome Required**: ❌ No
**Tests**: 20

Tests the CDP script validation system without needing Chrome.

```bash
cargo test --package robert-webdriver validation
```

---

### 🌐 E2E Tests (`e2e.rs`)
**Purpose**: End-to-end Chrome automation
**Chrome Required**: ✅ Yes (auto-downloads)
**Tests**: 3

All use CDP commands for reliable navigation with local test server.

- `test_navigate_and_get_title` - Navigate, extract title and text
- `test_get_element_text` - Extract h1 element text
- `test_get_page_source` - Get full page HTML source

```bash
cargo test --package robert-webdriver --test e2e -- --test-threads=1
```

---

### 🎯 Headless Integration Tests (`headless_integration.rs`)
**Purpose**: Headless CI/CD workflows
**Chrome Required**: ✅ Yes (auto-downloads)
**Tests**: 5

- `test_basic_navigation_headless` - Navigation + title via CDP
- `test_cdp_script_execution_headless` - Complete CDP script execution
- `test_screenshot_capture_headless` - Capture screenshot
- `test_data_extraction_headless` - Extract structured data
- `test_multiple_commands_headless` - Multiple CDP commands

```bash
cargo test --package robert-webdriver --test headless_integration -- --test-threads=1
```

---

### 💬 Chat UI Tests (`chat_ui_test.rs`)
**Purpose**: Test chat UI injection and messaging
**Chrome Required**: ✅ Yes (auto-downloads)
**Tests**: 7

All tests use local test server for complete isolation from external sites.

- `test_chat_ui_injection` - Verify automatic chat UI injection on navigation
- `test_send_agent_message` - Test sending messages from agent to chat
- `test_clear_chat_messages` - Test clearing all chat messages
- `test_chat_ui_persists_across_navigation` - Test chat UI re-injection on page navigation
- `test_chat_ui_collapse_expand` - Test collapse/expand functionality
- `test_chat_ui_can_be_disabled` - Test disabling automatic injection
- `test_manual_chat_ui_injection` - Test manual injection control

```bash
cargo test --package robert-webdriver --test chat_ui_test
```

---

### 🧪 CDP API Tests

#### CDP Execution Tests (`cdp_execution_test.rs`)
**Purpose**: Test ChromeDriver CDP API
**Chrome Required**: ✅ Yes (auto-downloads)
**Tests**: 4

- `test_cdp_page_access` - Verify CDP page access
- `test_cdp_navigation` - Test CDP navigation commands
- `test_send_cdp_command_evaluate` - Test send_cdp_command API with Runtime.evaluate
- `test_send_cdp_command_unsupported` - Test error handling for unsupported CDP commands

```bash
cargo test --package robert-webdriver --test cdp_execution_test
```

---

#### CDP Script Execution Tests (`cdp_script_execution_test.rs`)
**Purpose**: Test programmatic CDP script execution
**Chrome Required**: ✅ Yes (auto-downloads)
**Tests**: 5

- `test_execute_navigation_and_screenshot` - Navigate + screenshot
- `test_execute_data_extraction` - Extract title and heading
- `test_execute_programmatic_script` - Programmatic script creation
- `test_invalid_cdp_command` - Error handling for invalid commands
- `test_execute_cdp_script_from_file` - File-based script execution

```bash
cargo test --package robert-webdriver --test cdp_script_execution_test
```

---

#### CDP Generator Tests (`cdp_generator_test.rs`)
**Purpose**: Unit tests for CDP validation
**Chrome Required**: ❌ No
**Tests**: 6

Tests the CDP script validation system without Chrome or AI integration.

```bash
cargo test --package robert-webdriver --test cdp_generator_test
```

---

## Test Summary

| Category | Count | Chrome | Status |
|----------|-------|--------|--------|
| Meta/Infrastructure | 4 | ❌ | ✅ All Pass |
| Validation | 20 | ❌ | ✅ All Pass |
| Library Unit | 15 | ❌ | ✅ All Pass |
| Chat UI | 7 | ✅ | ✅ All Pass |
| CDP Execution | 4 | ✅ | ✅ All Pass |
| CDP Script Execution | 5 | ✅ | ✅ All Pass |
| CDP Generator/Validation | 6 | ❌ | ✅ All Pass |
| E2E | 3 | ✅ | ✅ All Pass |
| Headless Integration | 5 | ✅ | ✅ All Pass |
| **Total** | **69** | - | **✅ 69/69** |
| Doc Tests | 2 | ❌ | ✅ All Pass |
| **Grand Total** | **71** | - | **✅ 71/71** |

---

## Key Features

### Local Test Server
All Chrome tests use `test_server` module:
- Random ports for isolation
- Serves example.com-like HTML with multiple pages (/page2, /page3)
- `wait_ready()` for reliability
- Zero network dependencies
- Offline capable
- Navigation testing support with inter-page links

### CDP Commands
E2E tests use CDP commands directly (not high-level `navigate()`) because:
- More reliable with concurrent Chrome instances
- Better control over navigation
- Consistent across environments

### Test Organization Improvements
- ✅ Removed duplicate test_server tests (were running 3x)
- ✅ Created separate `meta_infrastructure_test.rs` for infrastructure tests
- ✅ Converted all ignored tests to use uniform patterns (local server, CDP commands)
- ✅ Removed external file dependencies from tests
- ✅ Separated AI-based generation tests from validation tests
- ✅ Converted ignored doc tests to proper integration tests
- ✅ Each test has clear, distinct purpose
- ✅ **All 53 tests now pass without any #[ignore] attributes**
- ✅ **All 2 doc tests pass (previously ignored)**
- ✅ **55 total tests passing**

---

## Quick Commands

```bash
# All tests (fully parallel, no restrictions!)
cargo test --package robert-webdriver

# Fast tests only (no Chrome, ~instant)
cargo test --package robert-webdriver validation
cargo test --package robert-webdriver --test meta_infrastructure_test

# Chrome tests only
cargo test --package robert-webdriver --test e2e
cargo test --package robert-webdriver --test headless_integration

# Linting
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## Test Isolation ✅

**Full parallel execution supported!** Each test gets:
- ✅ Own test server on random port
- ✅ Unique Chrome profile directory (timestamp-based)
- ✅ Clean Chrome instance with isolated user data
- ✅ Automatic cleanup on test completion

**No `--test-threads=1` required!** The previous issue with shared profile directories has been fixed by using unique temporary directories for each Chrome instance.
