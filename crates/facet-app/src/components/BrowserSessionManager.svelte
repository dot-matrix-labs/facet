<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';

  // Types matching Rust backend
  interface SessionInfo {
    id: { 0: string };
    profile_name: string;
    profile_type: string;
    created_at: string;
    headless: boolean;
  }

  interface LaunchBrowserResponse {
    session_id: string;
    session_info: SessionInfo;
  }

  interface CloseBrowserResponse {
    success: boolean;
    message: string;
  }

  interface BrowserStatusResponse {
    has_active_sessions: boolean;
    active_session_count: number;
    sessions: SessionInfo[];
  }

  // Component state
  let activeSessions: SessionInfo[] = [];
  let isLaunching = false;
  let isClosing = false;
  let errorMessage = '';
  let successMessage = '';
  let headlessMode = false;

  // Load initial status
  onMount(async () => {
    await refreshStatus();
    // Poll for status updates every 2 seconds
    statusInterval = setInterval(refreshStatus, 2000);
  });

  let statusInterval: ReturnType<typeof setInterval>;

  onDestroy(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
  });

  // Refresh browser status
  async function refreshStatus() {
    try {
      const status: BrowserStatusResponse = await invoke('get_browser_status');
      activeSessions = status.sessions || [];
    } catch (error) {
      console.error('Failed to get browser status:', error);
    }
  }

  // Launch a new browser session
  async function launchBrowser() {
    isLaunching = true;
    errorMessage = '';
    successMessage = '';

    try {
      const response: LaunchBrowserResponse = await invoke('launch_browser_session', {
        request: {
          headless: headlessMode,
          no_sandbox: false,
        },
      });

      successMessage = `Browser session launched successfully! Session ID: ${response.session_id.substring(0, 8)}...`;
      await refreshStatus();
    } catch (error) {
      errorMessage = `Failed to launch browser: ${error}`;
      console.error('Launch error:', error);
    } finally {
      isLaunching = false;
    }
  }

  // Close a specific browser session
  async function closeSession(sessionId: string) {
    isClosing = true;
    errorMessage = '';
    successMessage = '';

    try {
      const response: CloseBrowserResponse = await invoke('close_browser_session', {
        sessionId: sessionId,
      });

      if (response.success) {
        successMessage = response.message;
        await refreshStatus();
      }
    } catch (error) {
      errorMessage = `Failed to close session: ${error}`;
      console.error('Close error:', error);
    } finally {
      isClosing = false;
    }
  }

  // Close all browser sessions
  async function closeAllSessions() {
    isClosing = true;
    errorMessage = '';
    successMessage = '';

    try {
      const count: number = await invoke('close_all_browser_sessions');
      successMessage = `Closed ${count} session(s)`;
      await refreshStatus();
    } catch (error) {
      errorMessage = `Failed to close all sessions: ${error}`;
      console.error('Close all error:', error);
    } finally {
      isClosing = false;
    }
  }

  // Format date for display
  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleString();
  }

  // Get session ID string
  function getSessionId(session: SessionInfo): string {
    return typeof session.id === 'string' ? session.id : session.id['0'];
  }
</script>

<div class="browser-session-manager">
  <div class="header">
    <h2>Browser Sessions (Phase 2)</h2>
    <p class="description">Manage ephemeral browser sessions with automatic cleanup</p>
  </div>

  <!-- Launch Controls -->
  <div class="launch-controls">
    <div class="config-options">
      <label>
        <input type="checkbox" bind:checked={headlessMode} />
        Headless Mode
      </label>
    </div>

    <button
      class="btn btn-primary"
      on:click={launchBrowser}
      disabled={isLaunching || activeSessions.length >= 1}
    >
      {#if isLaunching}
        <span class="spinner"></span>
        Launching...
      {:else}
        Launch Browser Session
      {/if}
    </button>

    {#if activeSessions.length >= 1}
      <p class="info-text">Phase 2 limit: Only 1 active session at a time</p>
    {/if}
  </div>

  <!-- Status Messages -->
  {#if errorMessage}
    <div class="alert alert-error">
      <strong>Error:</strong>
      {errorMessage}
    </div>
  {/if}

  {#if successMessage}
    <div class="alert alert-success">
      {successMessage}
    </div>
  {/if}

  <!-- Active Sessions List -->
  <div class="sessions-list">
    <div class="sessions-header">
      <h3>Active Sessions ({activeSessions.length})</h3>
      {#if activeSessions.length > 0}
        <button
          class="btn btn-secondary btn-small"
          on:click={closeAllSessions}
          disabled={isClosing}
        >
          Close All
        </button>
      {/if}
    </div>

    {#if activeSessions.length === 0}
      <div class="empty-state">
        <p>No active browser sessions</p>
        <p class="hint">Click "Launch Browser Session" to start</p>
      </div>
    {:else}
      <div class="sessions-grid">
        {#each activeSessions as session}
          <div class="session-card">
            <div class="session-info">
              <div class="session-header">
                <span class="session-name">{session.profile_name}</span>
                <span class="badge badge-{session.profile_type}">
                  {session.profile_type}
                </span>
              </div>

              <div class="session-details">
                <div class="detail">
                  <strong>Session ID:</strong>
                  <code>{getSessionId(session).substring(0, 8)}...</code>
                </div>
                <div class="detail">
                  <strong>Created:</strong>
                  {formatDate(session.created_at)}
                </div>
                <div class="detail">
                  <strong>Mode:</strong>
                  {session.headless ? 'Headless' : 'Visible'}
                </div>
              </div>
            </div>

            <div class="session-actions">
              <button
                class="btn btn-danger btn-small"
                on:click={() => closeSession(getSessionId(session))}
                disabled={isClosing}
              >
                Close Session
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .browser-session-manager {
    padding: 1.5rem;
    max-width: 900px;
    margin: 0 auto;
  }

  .header {
    margin-bottom: 2rem;
  }

  .header h2 {
    margin: 0 0 0.5rem 0;
    color: #333;
  }

  .description {
    margin: 0;
    color: #666;
    font-size: 0.9rem;
  }

  .launch-controls {
    background: #f5f5f5;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
  }

  .config-options {
    margin-bottom: 1rem;
  }

  .config-options label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .info-text {
    margin: 0.75rem 0 0 0;
    font-size: 0.85rem;
    color: #666;
    font-style: italic;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #007bff;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #0056b3;
  }

  .btn-secondary {
    background: #6c757d;
    color: white;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #545b62;
  }

  .btn-danger {
    background: #dc3545;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #c82333;
  }

  .btn-small {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .alert {
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
  }

  .alert-error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }

  .alert-success {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
  }

  .sessions-list {
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 1.5rem;
  }

  .sessions-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .sessions-header h3 {
    margin: 0;
    color: #333;
  }

  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: #999;
  }

  .empty-state p {
    margin: 0.5rem 0;
  }

  .hint {
    font-size: 0.9rem;
    color: #bbb;
  }

  .sessions-grid {
    display: grid;
    gap: 1rem;
  }

  .session-card {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    padding: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    transition: box-shadow 0.2s;
  }

  .session-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .session-info {
    flex: 1;
  }

  .session-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.75rem;
  }

  .session-name {
    font-weight: 600;
    color: #333;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
  }

  .badge-ephemeral {
    background: #e3f2fd;
    color: #1976d2;
  }

  .badge-named {
    background: #f3e5f5;
    color: #7b1fa2;
  }

  .session-details {
    display: grid;
    gap: 0.5rem;
  }

  .detail {
    font-size: 0.875rem;
    color: #666;
  }

  .detail strong {
    color: #333;
    margin-right: 0.5rem;
  }

  code {
    background: #f5f5f5;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-family: 'Courier New', monospace;
    font-size: 0.85rem;
  }

  .session-actions {
    margin-left: 1rem;
  }
</style>
