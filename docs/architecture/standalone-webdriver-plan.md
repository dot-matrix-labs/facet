# Standalone Webdriver Extraction Plan

 This plan details the steps to decouple `robert-webdriver` from `robert-app` and establish it as a standalone service.

## User Review Required
> [!IMPORTANT]
> This is a breaking change for the build process. `robert-app` will no longer include webdriver capabilities out of the box. You must run `robert-webdriver` separately.

## Proposed Changes

### 1. Refactor `robert-webdriver`
*   [ ] **Convert to Binary**: Add `src/main.rs` to `crates/robert-webdriver`.
*   [ ] **Add HTTP Server**: Implement a `warp` or `axum` server in `robert-webdriver`.
    *   `GET /health`: Returns 200 OK if ready.
    *   `POST /inference`: Accepts inference requests and triggers the existing webdriver logic.
*   [ ] **CLI Args**: Add `clap` for port configuration (default to e.g., 3030 or similar).

### 2. Update `robert-app`
*   [ ] **Remove Dependency**: Remove `robert-webdriver` from `crates/robert-app/src-tauri/Cargo.toml`.
*   [ ] **Remove Direct Calls**: Delete/Refactor any Rust code in `robert-app` that directly calls `robert-webdriver`.
*   [ ] **Implement Discovery**:
    *   On startup/periodically, ping `http://localhost:<PORT>/health`.
*   [ ] **Update UI State**:
    *   Store connection status in Tauri state.
    *   Frontend queries this state to toggle "Debugging/Developer" sections for webdriver.
*   [ ] **Implement Client**:
    *   When user requests webdriving (chat interface), send JSON payload to `http://localhost:<PORT>/inference`.

### 3. Cleanup
*   [ ] **Workspace**: Ensure `robert-webdriver` is built as a binary in the root `Cargo.toml` default members (it already is, but ensure `robert-app` doesn't dep on it).

## Verification Plan

### Automated Tests
*   **Unit Tests**: Ensure `robert-webdriver` handlers are unit tested.
*   **Integration**:
    1.  Start `cargo run -p robert-webdriver`.
    2.  Run `curl http://localhost:<PORT>/health`.
    3.  Run `cargo run -p robert-app`.
    4.  Verify `robert-app` detects the running server.

### Manual Verification
1.  **Scenario A: Server Running**
    *   Start `robert-webdriver`.
    *   Open `robert-app`.
    *   Check Developer/Debug section -> Webdriver features should be **visible**.
    *   Send a request -> Should verify it hits the webdriver server logs.
2.  **Scenario B: Server Stopped**
    *   Stop `robert-webdriver`.
    *   Restart `robert-app` (or wait for polling).
    *   Check Developer/Debug section -> Webdriver features should be **hidden**.
