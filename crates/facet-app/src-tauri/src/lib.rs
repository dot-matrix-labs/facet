// Disable doctests for this crate
#![cfg_attr(doctest, allow(dead_code, unused_imports, unused_variables))]

// Modules
mod agent;
mod claude;
mod commands;
pub mod developer_mode;
mod events;
mod logging;
pub mod profiles;
mod state;

use state::AppState;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize custom logger that writes to both console and encrypted log file
    logging::logger::init_logger();

    log::info!("🚀 Facet App starting...");

    // Check if Claude CLI is accessible
    check_claude_cli_availability();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new())
        .setup(|app| {
            let state = app.state::<AppState>();
            let webdriver_mode = state.webdriver_mode.clone();

            // Spawn the embedded facet-server
            tauri::async_runtime::spawn(async move {
                log::info!("🚀 Starting embedded facet-server...");

                // load dev defaults for now
                let config = facet_server::Config::dev_default();

                // Spawn server in a separate task
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = facet_server::server::run(config).await {
                        log::error!("❌ Embedded server error: {}", e);
                    }
                });

                // Wait for server to be healthy
                let client = reqwest::Client::new();
                let health_url = "http://localhost:8443/api/v1/health";
                let mut retries = 0;
                let max_retries = 30; // 30 attempts * 500ms = 15 seconds

                while retries < max_retries {
                    match client.get(health_url).send().await {
                        Ok(res) => {
                            if res.status().is_success() {
                                log::info!(
                                    "✅ Embedded facet-server is healthy and reachable at {}",
                                    health_url
                                );
                                *webdriver_mode.lock().await = true; // Still using this flag to indicate "backend ready"
                                break;
                            }
                        }
                        Err(_) => {
                            // Server starting up...
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    retries += 1;
                }

                if retries >= max_retries {
                    log::error!("❌ Timed out waiting for embedded server to start");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Legacy commands removed/refactored
            commands::check_claude_health,
            commands::run_diagnostics,
            // commands::validate_cdp_script,
            // commands::validate_cdp_script_file,
            // commands::execute_cdp_script,
            // Developer mode commands
            commands::get_system_paths,
            commands::start_dev_test_server,
            commands::stop_dev_test_server,
            commands::get_dev_test_server_status,
            // Developer mode screenshot commands
            commands::dev_capture_screenshot,
            commands::dev_list_screenshots,
            commands::dev_delete_all_screenshots,
            commands::dev_delete_screenshot,
            // Agent workflow commands
            commands::process_chat_message,
            commands::init_agent_configs,
            commands::list_agent_configs,
            commands::get_agent_config,
            commands::update_agent_config,
            commands::submit_action_feedback,
            // Profile management commands
            commands::create_user,
            commands::login_user,
            commands::logout_user,
            commands::get_current_user,
            commands::list_users,
            commands::get_user_profile,
            commands::update_user_profile,
            commands::has_users,
            // Browser session management commands (Phase 2)
            commands::browser::launch_browser_session,
            commands::browser::close_browser_session,
            commands::browser::get_browser_status,
            commands::browser::close_all_browser_sessions,
            commands::browser::execute_webdriver_inference,
            // Command system commands (Phase 3 - Markdown-based)
            commands::save_command,
            commands::get_command,
            commands::list_commands,
            commands::delete_command,
            commands::build_command_prompt,
            commands::get_static_cdp,
            // Logging commands
            commands::log_frontend_message,
            commands::get_logs,
            commands::clear_logs,
            commands::get_log_size,
            // Feedback commands
            commands::submit_application_feedback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Check if Claude CLI is accessible for process spawning
fn check_claude_cli_availability() {
    use std::process::Command;

    log::info!("🔍 Checking Claude CLI availability...");

    match Command::new("claude").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                log::info!("✓ Claude CLI found: {}", version.trim());
            } else {
                log::warn!(
                    "⚠️  Claude CLI command failed with status: {}",
                    output.status
                );
                log::warn!("   Make sure Claude is installed and in your PATH");
                log::warn!("   Visit: https://www.anthropic.com/claude-cli");
            }
        }
        Err(e) => {
            log::warn!("⚠️  Claude CLI not found or not accessible: {}", e);
            log::warn!("   Agent workflows will not be available without Claude CLI");
            log::warn!("   Install from: https://www.anthropic.com/claude-cli");
        }
    }
}
