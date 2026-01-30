//! End-to-end tests for developer mode
//!
//! These tests verify the complete developer mode workflow including
//! command execution and state management

use facet_app_lib::developer_mode::{DevTestServer, SystemPaths};

#[tokio::test]
async fn test_complete_developer_workflow() {
    // Step 1: Start a test server (simulating start_dev_test_server command)
    let server = DevTestServer::start()
        .await
        .expect("Failed to start test server");

    server.wait_ready().await.expect("Server not ready");

    // Step 2: Get server URL (simulating get_dev_test_server_status command)
    let url = server.url();
    let port = server.port();

    assert!(url.starts_with("http://127.0.0.1:"));
    assert!(port > 0);

    // Step 3: Verify the test page is accessible
    let response = reqwest::get(&url).await.expect("Failed to fetch test page");

    assert!(response.status().is_success());
    let html = response.text().await.expect("Failed to read response");

    // Verify test page contains expected interactive elements
    assert!(html.contains("Facet Developer Test Page"));
    assert!(html.contains("test-input"));
    assert!(html.contains("test-textarea"));
    assert!(html.contains("test-button"));
    assert!(html.contains("alert-button"));

    // Step 4: Test the API endpoint
    let api_url = format!("{}/api/test", url);
    let api_response = reqwest::get(&api_url).await.expect("Failed to fetch API");

    assert!(api_response.status().is_success());

    let json: serde_json::Value = api_response
        .json()
        .await
        .expect("Failed to parse API response");

    assert_eq!(json["status"], "ok");

    // Step 5: Clean up (simulating stop_dev_test_server command)
    drop(server);

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify server is shut down
    let result = reqwest::get(&url).await;
    assert!(result.is_err(), "Server should be shut down");
}

#[tokio::test]
async fn test_manual_testing_workflow() {
    // This test simulates a developer's manual e2e testing workflow

    // 1. Start the test server
    let server = DevTestServer::start()
        .await
        .expect("Failed to start test server");

    server.wait_ready().await.expect("Server not ready");

    let url = server.url();

    // 2. Fetch the test page
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.expect("Failed to fetch page");

    assert!(response.status().is_success());

    // 3. Verify page structure for automation testing
    let html = response.text().await.expect("Failed to read HTML");

    // Check for input elements that would be used in CDP tests
    assert!(html.contains(r#"id="test-input""#));
    assert!(html.contains(r#"id="test-textarea""#));
    assert!(html.contains(r#"id="test-button""#));

    // Check for output element
    assert!(html.contains(r#"id="output""#));

    // 4. Verify API endpoint works
    let api_response = client
        .get(format!("{}/api/test", url))
        .send()
        .await
        .expect("Failed to fetch API");

    assert!(api_response.status().is_success());

    // 5. Clean up
    drop(server);
}

// #[tokio::test]
// async fn test_developer_mode_with_chrome_automation() {
//     // This test verifies the developer mode server works with Chrome automation
//     // FIXME: Update to use standalone webdriver service
//     /*
//     use facet_webdriver::{ChromeDriver, ConnectionMode};

//     // Start test server
//     let server = DevTestServer::start()
//         .await
//         .expect("Failed to start test server");

//     server.wait_ready().await.expect("Server not ready");

//     let url = server.url();

//     // Launch Chrome
//     let driver = ChromeDriver::new(ConnectionMode::Sandboxed {
//         chrome_path: None,
//         no_sandbox: true,
//         headless: true,
//     })
//     .await
//     .expect("Failed to launch Chrome");

//     // Navigate to test server
//     driver
//         .navigate(&url)
//         .await
//         .expect("Failed to navigate to test server");

//     // Get page title
//     let title = driver.title().await.expect("Failed to get title");
//     assert_eq!(title, "Facet Developer Test Page");

//     // Get page text
//     let text = driver
//         .get_page_text()
//         .await
//         .expect("Failed to get page text");
//     assert!(text.contains("Facet Developer Test Page"));
//     assert!(text.contains("Interactive Elements"));

//     // Clean up
//     driver.close().await.expect("Failed to close browser");
//     drop(server);
//     */
// }

#[test]
fn test_system_paths_structure() {
    // This test verifies SystemPaths contains all expected fields
    // We can't test actual values without a Tauri AppHandle, but we can verify the structure

    use serde_json;

    // Create a sample SystemPaths
    let paths = SystemPaths {
        installation_dir: "/test/installation".to_string(),
        config_dir: "/test/config".to_string(),
        data_dir: "/test/data".to_string(),
        cache_dir: "/test/cache".to_string(),
        temp_dir: "/test/temp".to_string(),
        current_dir: "/test/current".to_string(),
        chrome_path: Some("/test/chrome".to_string()),
    };

    // Verify it can be serialized
    let json = serde_json::to_string(&paths).expect("Failed to serialize");
    assert!(json.contains("installation_dir"));
    assert!(json.contains("config_dir"));
    assert!(json.contains("data_dir"));
    assert!(json.contains("cache_dir"));
    assert!(json.contains("temp_dir"));
    assert!(json.contains("current_dir"));
    assert!(json.contains("chrome_path"));

    // Verify it can be deserialized
    let deserialized: SystemPaths = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.installation_dir, paths.installation_dir);
    assert_eq!(deserialized.chrome_path, paths.chrome_path);
}

#[tokio::test]
async fn test_multiple_developer_sessions() {
    // Test that multiple developers can work independently

    let server1 = DevTestServer::start()
        .await
        .expect("Failed to start server 1");
    let server2 = DevTestServer::start()
        .await
        .expect("Failed to start server 2");

    server1.wait_ready().await.expect("Server 1 not ready");
    server2.wait_ready().await.expect("Server 2 not ready");

    // Verify different ports
    assert_ne!(server1.port(), server2.port());

    // Both should work independently
    let response1 = reqwest::get(&server1.url()).await.expect("Server 1 failed");
    let response2 = reqwest::get(&server2.url()).await.expect("Server 2 failed");

    assert!(response1.status().is_success());
    assert!(response2.status().is_success());

    // Clean up
    drop(server1);
    drop(server2);
}
