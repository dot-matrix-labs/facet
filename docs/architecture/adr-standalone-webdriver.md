# ADR: Standalone Webdriver Process

## Status
Proposed

## Context
Currently, the `robert-webdriver` functionality is integrated directly into the `robert-app` (Tauri) process or linked as a library. This tightly couples the desktop application with the webdriver logic. 

A major architectural change is required where the desktop app will **not** manage the webdriver within its own process. The desktop app should only serve as a chat interface for general-purpose inference requests. The webdriver should be a separate binary that, when started, serves a local HTTP server.

## Decision
1.  **Separate Binary**: `robert-webdriver` will be refactored into a standalone executable binary.
2.  **HTTP Server Interface**: The `robert-webdriver` binary will host a local HTTP server.
    *   It will accept requests (e.g., from the desktop app).
    *   It will execute inference workflows for generating webdriving actions.
3.  **Decoupling**: `robert-app` (Tauri) will **not** compile `robert-webdriver`, import any of its modules, or have any Rust bindings to it.
4.  **Discovery**: `robert-app` will merely search for the `robert-webdriver` server (e.g., by checking a local port or health endpoint).
5.  **UI Integration**:
    *   If the server is found/healthy: Show webdriving features in the debugging and developer section.
    *   If not found: Do not display any information about the webdriver.

## Consequences
### Positive
*   **Decoupling**: logical separation of concerns. `robert-app` becomes lighter and focused on chat/UI.
*   **Stability**: Crashes or issues in the webdriver process do not directly crash the main desktop app.
*   **Flexibility**: The webdriver can be updated, restarted, or run independently of the UI.
*   **Build Times**: `robert-app` compile times should decrease as it no longer builds the webdriver stack.

### Negative
*   **Complexity**: Requires managing two separate processes.
*   **User Experience**: User needs to ensure the webdriver binary is running for features to appear (or we need a separate launcher/manager, though the prompt implies the user manually starts it or it's external).
*   **IPC Overhead**: Communication is now over HTTP instead of direct function calls (negligible for this use case).

## Technical Implementation Details
*   `robert-webdriver` will add `src/main.rs` and depend on a web framework (likely `warp` or `axum`).
*   Definition of a clear HTTP API (e.g., `POST /v1/inference`, `GET /health`).
*   `robert-app` will use `reqwest` to query the local webdriver server.
