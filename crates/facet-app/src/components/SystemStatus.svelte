<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface ClaudeHealthCheck {
    installed: boolean;
    path?: string;
    version?: string;
    authenticated: boolean;
    issues: string[];
    suggestions: string[];
    status: 'healthy' | 'warning' | 'error';
  }

  interface SystemDiagnostics {
    chrome_status: string;
    chrome_installed: boolean;
    claude_health: ClaudeHealthCheck;
    browser_running: boolean;
    current_url?: string;
  }

  let diagnostics: SystemDiagnostics | null = null;
  let loading = true;
  let expanded = false;

  onMount(async () => {
    await checkHealth();
  });

  async function checkHealth() {
    loading = true;
    try {
      diagnostics = await invoke<SystemDiagnostics>('run_diagnostics');
    } catch (error) {
      console.error('Failed to run diagnostics:', error);
    } finally {
      loading = false;
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'healthy':
        return '#4caf50';
      case 'warning':
        return '#ff9800';
      case 'error':
        return '#f44336';
      default:
        return '#999';
    }
  }

  function getStatusIcon(status: string): string {
    switch (status) {
      case 'healthy':
        return '✓';
      case 'warning':
        return '⚠';
      case 'error':
        return '✗';
      default:
        return '○';
    }
  }
</script>

<div class="system-status">
  <div
    class="status-header"
    role="button"
    tabindex="0"
    on:click={() => (expanded = !expanded)}
    on:keydown={(e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        expanded = !expanded;
      }
    }}
  >
    <h4>System Status</h4>
    <button class="refresh-btn" on:click|stopPropagation={checkHealth} disabled={loading}>
      {loading ? '⟳' : '↻'} Refresh
    </button>
  </div>

  {#if loading}
    <div class="loading">Checking system status...</div>
  {:else if diagnostics}
    <div class="status-grid">
      <!-- Chrome Status -->
      <div class="status-item">
        <span class="status-label">Chrome Browser</span>
        <span
          class="status-value"
          class:running={diagnostics.browser_running}
          class:stopped={!diagnostics.browser_running}
        >
          {diagnostics.chrome_status}
        </span>
      </div>

      <!-- Claude Status -->
      <div class="status-item claude-status">
        <span class="status-label">Claude CLI</span>
        <span
          class="status-value"
          style="color: {getStatusColor(diagnostics.claude_health.status)}"
        >
          {getStatusIcon(diagnostics.claude_health.status)}
          {diagnostics.claude_health.status === 'healthy'
            ? 'Ready'
            : diagnostics.claude_health.status === 'warning'
              ? 'Warning'
              : 'Not Available'}
        </span>
      </div>

      <!-- Current URL (if browser running) -->
      {#if diagnostics.browser_running && diagnostics.current_url}
        <div class="status-item current-url">
          <span class="status-label">Current Page</span>
          <span class="status-value url">{diagnostics.current_url}</span>
        </div>
      {/if}
    </div>

    <!-- Expanded Details -->
    {#if expanded}
      <div class="details">
        <div class="detail-section">
          <h5>Claude CLI Details</h5>
          {#if diagnostics.claude_health.status === 'healthy'}
            <div class="detail-item success">
              <strong>Status:</strong> Ready to use
            </div>
            <div class="detail-item">
              <strong>Version:</strong>
              {diagnostics.claude_health.version || 'Unknown'}
            </div>
            <div class="detail-item">
              <strong>Path:</strong>
              {diagnostics.claude_health.path || 'Unknown'}
            </div>
            <div class="detail-item">
              <strong>Authenticated:</strong>
              {diagnostics.claude_health.authenticated ? 'Yes ✓' : 'No ✗'}
            </div>
          {:else}
            <div class="detail-item error">
              <strong>Issues:</strong>
              <ul>
                {#each diagnostics.claude_health.issues as issue}
                  <li>{issue}</li>
                {/each}
              </ul>
            </div>
            <div class="detail-item warning">
              <strong>Suggestions:</strong>
              <ul>
                {#each diagnostics.claude_health.suggestions as suggestion}
                  <li>{suggestion}</li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>

        {#if diagnostics.claude_health.status !== 'healthy'}
          <div class="setup-instructions">
            <h5>Setup Instructions</h5>
            <ol>
              {#if !diagnostics.claude_health.installed}
                <li>
                  Install Claude CLI:
                  <code>npm install -g @anthropic-ai/claude-code</code>
                  <br />
                  or
                  <code>brew install claude</code>
                </li>
              {/if}
              {#if diagnostics.claude_health.installed && !diagnostics.claude_health.authenticated}
                <li>
                  Authenticate Claude CLI:
                  <code>claude setup-token</code>
                </li>
              {/if}
              <li>Refresh this status panel to verify installation</li>
            </ol>
          </div>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .system-status {
    background: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .status-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    cursor: pointer;
    user-select: none;
  }

  .status-header h4 {
    margin: 0;
    font-size: 1rem;
    color: #d4d4d4;
  }

  .refresh-btn {
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
    background: #3a3a3a;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .refresh-btn:hover:not(:disabled) {
    background: #4a4a4a;
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading {
    text-align: center;
    padding: 1rem;
    color: #999;
    font-style: italic;
  }

  .status-grid {
    display: grid;
    gap: 0.75rem;
  }

  .status-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: #1e1e1e;
    border-radius: 4px;
    border-left: 3px solid transparent;
  }

  .status-label {
    font-size: 0.9rem;
    color: #a0a0a0;
  }

  .status-value {
    font-size: 0.9rem;
    font-weight: 500;
  }

  .status-value.running {
    color: #4caf50;
  }

  .status-value.stopped {
    color: #999;
  }

  .status-value.url {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #4a90e2;
    font-size: 0.85rem;
  }

  .claude-status {
    border-left-color: #9b59b6;
  }

  .details {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #3a3a3a;
  }

  .detail-section {
    margin-bottom: 1rem;
  }

  .detail-section h5 {
    margin: 0 0 0.75rem 0;
    font-size: 0.95rem;
    color: #d4d4d4;
  }

  .detail-item {
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
    color: #a0a0a0;
    line-height: 1.4;
  }

  .detail-item strong {
    color: #d4d4d4;
  }

  .detail-item.success {
    color: #4caf50;
  }

  .detail-item.error {
    color: #f44336;
  }

  .detail-item.warning {
    color: #ff9800;
  }

  .detail-item ul {
    margin: 0.25rem 0 0 1rem;
    padding: 0;
  }

  .detail-item li {
    margin: 0.25rem 0;
  }

  .setup-instructions {
    background: #1a1a2e;
    padding: 1rem;
    border-radius: 4px;
    border: 1px solid #3a3a5a;
  }

  .setup-instructions h5 {
    margin: 0 0 0.75rem 0;
    color: #4a90e2;
    font-size: 0.95rem;
  }

  .setup-instructions ol {
    margin: 0;
    padding-left: 1.5rem;
    color: #a0a0a0;
    font-size: 0.85rem;
  }

  .setup-instructions li {
    margin: 0.5rem 0;
    line-height: 1.6;
  }

  .setup-instructions code {
    background: #0d1117;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 0.8rem;
    color: #79c0ff;
    display: inline-block;
    margin: 0.25rem 0;
  }
</style>
