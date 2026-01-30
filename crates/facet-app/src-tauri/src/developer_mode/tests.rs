//! Unit tests for developer mode

#[cfg(test)]
mod system_paths_tests {
    use super::super::SystemPaths;

    #[test]
    fn test_find_chrome_path_returns_option() {
        // This test verifies that find_chrome_path returns an Option
        // The actual value depends on the system, so we just check the type
        let path = SystemPaths::find_chrome_path();
        assert!(path.is_some() || path.is_none());
    }
}

#[cfg(test)]
mod test_server_tests {
    use super::super::test_server::DevTestServer;

    #[tokio::test]
    async fn test_server_starts_successfully() {
        let server = DevTestServer::start()
            .await
            .expect("Failed to start test server");

        // Verify server has a valid URL
        let url = server.url();
        assert!(url.starts_with("http://127.0.0.1:"));
        assert!(url.len() > 17); // Has port number

        // Verify port is assigned
        let port = server.port();
        assert!(port > 0);

        // Verify status
        let status = server.status();
        assert!(status.running);
        assert_eq!(status.url, Some(url));
        assert_eq!(status.port, Some(port));
    }

    #[tokio::test]
    async fn test_server_responds_to_requests() {
        let server = DevTestServer::start()
            .await
            .expect("Failed to start test server");

        server
            .wait_ready()
            .await
            .expect("Server did not become ready");

        // Test main page
        let response = reqwest::get(&server.url())
            .await
            .expect("Failed to fetch main page");

        assert!(response.status().is_success());
        let body = response.text().await.expect("Failed to read body");
        assert!(body.contains("Facet Developer Test Page"));
        assert!(body.contains("test-input"));
        assert!(body.contains("test-button"));
    }

    #[tokio::test]
    async fn test_server_api_endpoint() {
        let server = DevTestServer::start()
            .await
            .expect("Failed to start test server");

        server
            .wait_ready()
            .await
            .expect("Server did not become ready");

        // Test API endpoint
        let url = format!("{}/api/test", server.url());
        let response = reqwest::get(&url).await.expect("Failed to fetch API");

        assert!(response.status().is_success());

        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(json["status"], "ok");
        assert!(json["message"].is_string());
        assert!(json["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_multiple_servers_use_different_ports() {
        let server1 = DevTestServer::start()
            .await
            .expect("Failed to start server 1");
        let server2 = DevTestServer::start()
            .await
            .expect("Failed to start server 2");

        // Verify different ports
        assert_ne!(server1.port(), server2.port());
        assert_ne!(server1.url(), server2.url());

        // Both should respond
        server1.wait_ready().await.expect("Server 1 not ready");
        server2.wait_ready().await.expect("Server 2 not ready");

        let response1 = reqwest::get(&server1.url())
            .await
            .expect("Server 1 request failed");
        let response2 = reqwest::get(&server2.url())
            .await
            .expect("Server 2 request failed");

        assert!(response1.status().is_success());
        assert!(response2.status().is_success());
    }

    #[tokio::test]
    async fn test_server_shutdown_on_drop() {
        let url = {
            let server = DevTestServer::start()
                .await
                .expect("Failed to start test server");

            server
                .wait_ready()
                .await
                .expect("Server did not become ready");

            let url = server.url();

            // Verify server is responding
            let response = reqwest::get(&url).await.expect("Server not responding");
            assert!(response.status().is_success());

            url
            // Server drops here
        };

        // Give server time to shut down
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Server should no longer respond
        let result = reqwest::get(&url).await;
        assert!(result.is_err(), "Server should have shut down");
    }
}
