import { invoke } from '@tauri-apps/api/core';
import type {
  NavigationResult,
  SystemPaths,
  TestServerStatus,
  ScreenshotInfo,
  ExecutionReport,
  ChatMessageRequest,
  WorkflowResult,
  ProfileResult,
  Command,
  CommandInfo,
  JsonValue,
  // Legacy types (deprecated)
  // Legacy types (deprecated)
  CommandConfig,
  WebdriverStatus,
} from './types';

export async function launchBrowser(screenWidth?: number, screenHeight?: number): Promise<string> {
  return await invoke<string>('launch_browser', {
    screenWidth: screenWidth ? screenWidth : null,
    screenHeight: screenHeight ? screenHeight : null,
  });
}

export async function getBrowserStatus(): Promise<WebdriverStatus> {
  return await invoke<WebdriverStatus>('get_browser_status');
}

export async function navigateToUrl(url: string): Promise<NavigationResult> {
  return await invoke<NavigationResult>('navigate_to_url', { url });
}

export async function getPageContent(): Promise<string> {
  return await invoke<string>('get_page_content');
}

export async function closeBrowser(): Promise<void> {
  return await invoke<void>('close_browser');
}

// Developer mode commands
export async function getSystemPaths(): Promise<SystemPaths> {
  return await invoke<SystemPaths>('get_system_paths');
}

export async function startDevTestServer(): Promise<TestServerStatus> {
  return await invoke<TestServerStatus>('start_dev_test_server');
}

export async function stopDevTestServer(): Promise<TestServerStatus> {
  return await invoke<TestServerStatus>('stop_dev_test_server');
}

export async function getDevTestServerStatus(): Promise<TestServerStatus> {
  return await invoke<TestServerStatus>('get_dev_test_server_status');
}

// Screenshot management commands
export async function devCaptureScreenshot(): Promise<ScreenshotInfo> {
  return await invoke<ScreenshotInfo>('dev_capture_screenshot');
}

export async function devListScreenshots(): Promise<ScreenshotInfo[]> {
  return await invoke<ScreenshotInfo[]>('dev_list_screenshots');
}

export async function devDeleteAllScreenshots(): Promise<number> {
  return await invoke<number>('dev_delete_all_screenshots');
}

export async function devDeleteScreenshot(path: string): Promise<void> {
  return await invoke<void>('dev_delete_screenshot', { path });
}

// CDP script execution
export async function executeCdpScript(scriptJson: string): Promise<ExecutionReport> {
  return await invoke<ExecutionReport>('execute_cdp_script', { scriptJson });
}

// Agent workflow commands
export async function processChatMessage(request: ChatMessageRequest): Promise<WorkflowResult> {
  return await invoke<WorkflowResult>('process_chat_message', { request });
}

// ============================================================================
// Command System Commands (Phase 3 - Markdown-based)
// ============================================================================

/**
 * Save a markdown-based command
 * @param command - The Command structure to save
 * @returns Result indicating success or error
 */
export async function saveCommand(command: Command): Promise<ProfileResult<void>> {
  return await invoke<ProfileResult<void>>('save_command', { command });
}

/**
 * Get a command by name
 * @param name - The command identifier (kebab-case)
 * @returns The complete Command structure
 */
export async function getCommand(name: string): Promise<ProfileResult<Command>> {
  return await invoke<ProfileResult<Command>>('get_command', { name });
}

/**
 * List all saved commands
 * @returns Array of CommandInfo summaries
 */
export async function listCommands(): Promise<ProfileResult<CommandInfo[]>> {
  return await invoke<ProfileResult<CommandInfo[]>>('list_commands');
}

/**
 * Delete a command by name
 * @param name - The command identifier to delete
 * @returns Result indicating success or error
 */
export async function deleteCommand(name: string): Promise<ProfileResult<void>> {
  return await invoke<ProfileResult<void>>('delete_command', { name });
}

/**
 * Build AI prompt for dynamic CDP generation
 * @param name - The command identifier
 * @param parameters - User-provided parameter values
 * @returns AI prompt string ready to send to Claude/OpenAI
 */
export async function buildCommandPrompt(
  name: string,
  parameters: Record<string, JsonValue>
): Promise<ProfileResult<string>> {
  return await invoke<ProfileResult<string>>('build_command_prompt', { name, parameters });
}

/**
 * Get static CDP script from markdown template (fallback)
 * @param name - The command identifier
 * @param parameters - User-provided parameter values for substitution
 * @returns Static CDP script JSON if available, null otherwise
 */
export async function getStaticCdp(
  name: string,
  parameters: Record<string, JsonValue>
): Promise<ProfileResult<string | null>> {
  return await invoke<ProfileResult<string | null>>('get_static_cdp', { name, parameters });
}

// ============================================================================
// Legacy Command API (JSON-based - DEPRECATED)
// ============================================================================

/**
 * @deprecated Use the new markdown-based saveCommand instead
 * Save a legacy JSON command configuration
 */
export async function saveLegacyCommand(config: CommandConfig): Promise<ProfileResult<void>> {
  return await invoke<ProfileResult<void>>('save_legacy_command', { config });
}

/**
 * @deprecated Use buildCommandPrompt or getStaticCdp instead
 * Execute a legacy command with parameters
 */
export async function executeCommand(
  name: string,
  params: Record<string, string>
): Promise<ProfileResult<string>> {
  return await invoke<ProfileResult<string>>('execute_command', { name, params });
}
