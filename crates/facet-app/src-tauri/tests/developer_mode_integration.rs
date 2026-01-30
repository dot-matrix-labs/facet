//! Integration tests for developer mode
//!
//! These tests verify the interaction between developer mode components
//! and the application state management.

use std::sync::Arc;
use tokio::sync::Mutex;

// Note: We can't easily test Tauri commands in integration tests without a full Tauri app instance,
// but we can test the underlying functionality

#[tokio::test]
async fn test_dev_server_lifecycle_in_state() {
    // This test simulates how the app state manages the dev server
    use facet_app_lib::developer_mode::DevTestServer;

    // Simulate app state with dev server
    let dev_server: Arc<Mutex<Option<DevTestServer>>> = Arc::new(Mutex::new(None));

    // Start server
    {
        let mut server_lock = dev_server.lock().await;
        assert!(server_lock.is_none());

        let server = DevTestServer::start()
            .await
            .expect("Failed to start server");

        server.wait_ready().await.expect("Server not ready");

        let url = server.url();
        let port = server.port();

        *server_lock = Some(server);

        // Verify we can get status while holding lock
        if let Some(s) = server_lock.as_ref() {
            assert_eq!(s.url(), url);
            assert_eq!(s.port(), port);
        }
    }

    // Verify server is still accessible after releasing lock
    {
        let server_lock = dev_server.lock().await;
        assert!(server_lock.is_some());

        if let Some(s) = server_lock.as_ref() {
            let url = s.url();
            let response = reqwest::get(&url).await.expect("Server not responding");
            assert!(response.status().is_success());
        }
    }

    // Stop server
    {
        let mut server_lock = dev_server.lock().await;
        let old_url = server_lock
            .as_ref()
            .map(|s| s.url())
            .expect("Server should exist");

        *server_lock = None;

        // Give server time to shut down
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify server is shut down
        let result = reqwest::get(&old_url).await;
        assert!(result.is_err(), "Server should be shut down");
    }
}

#[tokio::test]
async fn test_concurrent_access_to_dev_server_state() {
    use facet_app_lib::developer_mode::DevTestServer;

    let dev_server: Arc<Mutex<Option<DevTestServer>>> = Arc::new(Mutex::new(None));

    // Start server
    {
        let mut server_lock = dev_server.lock().await;
        let server = DevTestServer::start()
            .await
            .expect("Failed to start server");
        server.wait_ready().await.expect("Server not ready");
        *server_lock = Some(server);
    }

    // Spawn multiple tasks that read server status concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let dev_server_clone = Arc::clone(&dev_server);
        let handle = tokio::spawn(async move {
            let server_lock = dev_server_clone.lock().await;
            if let Some(s) = server_lock.as_ref() {
                let url = s.url();
                let response = reqwest::get(&url)
                    .await
                    .unwrap_or_else(|_| panic!("Task {} failed to fetch", i));
                assert!(response.status().is_success());
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    // Clean up
    let mut server_lock = dev_server.lock().await;
    *server_lock = None;
}

#[tokio::test]
async fn test_restart_dev_server() {
    use facet_app_lib::developer_mode::DevTestServer;

    let dev_server: Arc<Mutex<Option<DevTestServer>>> = Arc::new(Mutex::new(None));

    // Start, stop, and restart cycle
    for iteration in 1..=3 {
        // Start server
        let url = {
            let mut server_lock = dev_server.lock().await;
            let server = DevTestServer::start()
                .await
                .unwrap_or_else(|_| panic!("Failed to start server in iteration {}", iteration));
            server.wait_ready().await.expect("Server not ready");

            let url = server.url();

            // Verify it works
            let response = reqwest::get(&url).await.expect("Server not responding");
            assert!(response.status().is_success());

            *server_lock = Some(server);
            url
        };

        // Stop server
        {
            let mut server_lock = dev_server.lock().await;
            *server_lock = None;

            // Give server time to shut down
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Verify it's shut down
            let result = reqwest::get(&url).await;
            assert!(
                result.is_err(),
                "Server should be shut down in iteration {}",
                iteration
            );
        }
    }
}
