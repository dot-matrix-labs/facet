//! Developer mode test server
//!
//! Provides a local HTTP server for manual e2e testing of browser automation

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use warp::Filter;

/// Test server for developer mode manual testing
pub struct DevTestServer {
    addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

/// Test server status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestServerStatus {
    pub running: bool,
    pub url: Option<String>,
    pub port: Option<u16>,
}

impl DevTestServer {
    /// Start a new test server on a random available port
    pub async fn start() -> anyhow::Result<Self> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Routes - serve a more interactive test page for manual testing
        let index = warp::path::end().map(|| {
            warp::reply::html(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Facet Developer Test Page</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
            line-height: 1.6;
        }
        h1 { color: #667eea; }
        .section {
            background: #f5f5f5;
            padding: 20px;
            margin: 20px 0;
            border-radius: 8px;
        }
        button {
            background: #667eea;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            margin: 5px;
        }
        button:hover { background: #5568d3; }
        input, textarea {
            width: 100%;
            padding: 10px;
            margin: 10px 0;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
        }
        #output {
            background: white;
            padding: 15px;
            border: 1px solid #ddd;
            border-radius: 4px;
            min-height: 100px;
            margin-top: 10px;
        }
        .test-element { margin: 10px 0; }
    </style>
</head>
<body>
    <h1>🤖 Facet Developer Test Page</h1>
    <p>This page is for manual e2e testing of browser automation with CDP commands.</p>

    <div class="section">
        <h2>Interactive Elements</h2>
        <div class="test-element">
            <input id="test-input" type="text" placeholder="Test input field">
        </div>
        <div class="test-element">
            <textarea id="test-textarea" placeholder="Test textarea"></textarea>
        </div>
        <div class="test-element">
            <button id="test-button" onclick="handleButtonClick()">Click Me</button>
            <button id="alert-button" onclick="alert('Test Alert!')">Show Alert</button>
        </div>
    </div>

    <div class="section">
        <h2>Test Output</h2>
        <div id="output">No actions yet...</div>
    </div>

    <div class="section">
        <h2>Page Information</h2>
        <ul id="page-info">
            <li>URL: <span id="url"></span></li>
            <li>Title: <span id="title"></span></li>
            <li>User Agent: <span id="user-agent"></span></li>
        </ul>
    </div>

    <script>
        // Display page information
        document.getElementById('url').textContent = window.location.href;
        document.getElementById('title').textContent = document.title;
        document.getElementById('user-agent').textContent = navigator.userAgent;

        // Handle button clicks
        function handleButtonClick() {
            const input = document.getElementById('test-input').value;
            const textarea = document.getElementById('test-textarea').value;
            const output = document.getElementById('output');

            output.innerHTML = `
                <strong>Button Clicked!</strong><br>
                Input value: ${input || '(empty)'}<br>
                Textarea value: ${textarea || '(empty)'}<br>
                Timestamp: ${new Date().toISOString()}
            `;
        }

        // Log input changes
        document.getElementById('test-input').addEventListener('input', (e) => {
            console.log('Input changed:', e.target.value);
        });

        document.getElementById('test-textarea').addEventListener('input', (e) => {
            console.log('Textarea changed:', e.target.value);
        });
    </script>
</body>
</html>"#,
            )
        });

        // API endpoint to test JSON responses
        let api_test = warp::path("api").and(warp::path("test")).map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "message": "Test API endpoint",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        });

        let routes = index.or(api_test);

        // Bind to random port on localhost
        let (addr, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
                shutdown_rx.await.ok();
            });

        // Spawn server in background
        tokio::spawn(server);

        Ok(Self {
            addr,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    /// Get the base URL for this server (e.g., "http://127.0.0.1:12345")
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Get the port number
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Get server status
    pub fn status(&self) -> TestServerStatus {
        TestServerStatus {
            running: true,
            url: Some(self.url()),
            port: Some(self.port()),
        }
    }

    /// Wait for the server to be ready by making a test request
    pub async fn wait_ready(&self) -> anyhow::Result<()> {
        let url = self.url();
        let max_attempts = 10;

        for attempt in 1..=max_attempts {
            match reqwest::get(&url).await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    if attempt < max_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }

        anyhow::bail!(
            "Server did not become ready after {} attempts",
            max_attempts
        )
    }
}

impl Drop for DevTestServer {
    fn drop(&mut self) {
        // Signal server to shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_and_responds() {
        let server = DevTestServer::start()
            .await
            .expect("Failed to start server");
        server.wait_ready().await.expect("Server not ready");

        let response = reqwest::get(&server.url()).await.expect("Failed to fetch");

        assert!(response.status().is_success());
        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("Facet Developer Test Page"));
    }

    #[tokio::test]
    async fn test_server_api_endpoint() {
        let server = DevTestServer::start()
            .await
            .expect("Failed to start server");
        server.wait_ready().await.expect("Server not ready");

        let url = format!("{}/api/test", server.url());
        let response = reqwest::get(&url).await.expect("Failed to fetch");

        assert!(response.status().is_success());
        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(json["status"], "ok");
    }
}
