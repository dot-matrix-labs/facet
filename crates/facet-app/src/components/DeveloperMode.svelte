<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    getSystemPaths,
    startDevTestServer,
    stopDevTestServer,
    getDevTestServerStatus,
    launchBrowser,
    navigateToUrl,
    devCaptureScreenshot,
    devListScreenshots,
    devDeleteAllScreenshots,
    devDeleteScreenshot,
  } from '../lib/tauri';
  import type { SystemPaths, TestServerStatus, ScreenshotInfo } from '../lib/types';
  import CdpDebugger from './CdpDebugger.svelte';

  let systemPaths: SystemPaths | null = null;
  let serverStatus: TestServerStatus = { running: false, url: null, port: null };
  let screenshots: ScreenshotInfo[] = [];
  let loading = false;
  let screenshotLoading = false;
  let error: string | null = null;
  let statusCheckInterval: number | null = null;

  onMount(async () => {
    await loadSystemPaths();
    await checkServerStatus();
    await loadScreenshots();

    // Poll server status every 2 seconds
    statusCheckInterval = window.setInterval(async () => {
      await checkServerStatus();
    }, 2000);
  });

  onDestroy(() => {
    if (statusCheckInterval) {
      clearInterval(statusCheckInterval);
    }
  });

  async function loadSystemPaths() {
    try {
      loading = true;
      error = null;
      systemPaths = await getSystemPaths();
    } catch (e) {
      error = `Failed to load system paths: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function checkServerStatus() {
    try {
      serverStatus = await getDevTestServerStatus();
    } catch (e) {
      console.error('Failed to check server status:', e);
    }
  }

  async function handleStartServer() {
    try {
      loading = true;
      error = null;
      serverStatus = await startDevTestServer();
    } catch (e) {
      error = `Failed to start server: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function handleStopServer() {
    try {
      loading = true;
      error = null;
      serverStatus = await stopDevTestServer();
    } catch (e) {
      error = `Failed to stop server: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  async function handleOpenInBrowser() {
    if (!serverStatus.url) return;

    try {
      loading = true;
      error = null;

      // Ensure browser is launched
      await launchBrowser();

      // Navigate to test server
      await navigateToUrl(serverStatus.url);
    } catch (e) {
      error = `Failed to open in browser: ${e}`;
      console.error(error);
    } finally {
      loading = false;
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  // Screenshot management
  async function loadScreenshots() {
    try {
      screenshots = await devListScreenshots();
    } catch (e) {
      console.error('Failed to load screenshots:', e);
    }
  }

  async function handleCaptureScreenshot() {
    try {
      screenshotLoading = true;
      error = null;
      await devCaptureScreenshot();
      await loadScreenshots();
    } catch (e) {
      error = `Failed to capture screenshot: ${e}`;
      console.error(error);
    } finally {
      screenshotLoading = false;
    }
  }

  async function handleDeleteScreenshot(path: string) {
    if (!confirm('Delete this screenshot?')) return;

    try {
      screenshotLoading = true;
      error = null;
      await devDeleteScreenshot(path);
      await loadScreenshots();
    } catch (e) {
      error = `Failed to delete screenshot: ${e}`;
      console.error(error);
    } finally {
      screenshotLoading = false;
    }
  }

  async function handleDeleteAllScreenshots() {
    if (!confirm('Delete all screenshots? This cannot be undone.')) return;

    try {
      screenshotLoading = true;
      error = null;
      const count = await devDeleteAllScreenshots();
      screenshots = [];
      console.log(`Deleted ${count} screenshots`);
    } catch (e) {
      error = `Failed to delete screenshots: ${e}`;
      console.error(error);
    } finally {
      screenshotLoading = false;
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function formatSize(sizeKb: number): string {
    if (sizeKb < 1024) {
      return `${sizeKb} KB`;
    }
    return `${(sizeKb / 1024).toFixed(2)} MB`;
  }
</script>

<div class="developer-mode">
  <h2>🛠️ Developer Mode</h2>

  {#if error}
    <div class="error-banner">
      {error}
    </div>
  {/if}

  <!-- System Paths Section -->
  <section class="section">
    <h3>📁 System Paths</h3>
    {#if loading && !systemPaths}
      <div class="loading">Loading system paths...</div>
    {:else if systemPaths}
      <div class="paths-grid">
        <div class="path-item">
          <span class="path-label">Installation Directory:</span>
          <div class="path-value">
            <code>{systemPaths.installation_dir}</code>
            <button
              class="copy-btn"
              on:click={() => copyToClipboard(systemPaths!.installation_dir)}
            >
              📋
            </button>
          </div>
        </div>

        <div class="path-item">
          <span class="path-label">Config Directory:</span>
          <div class="path-value">
            <code>{systemPaths.config_dir}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.config_dir)}>
              📋
            </button>
          </div>
        </div>

        <div class="path-item">
          <span class="path-label">Data Directory:</span>
          <div class="path-value">
            <code>{systemPaths.data_dir}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.data_dir)}>
              📋
            </button>
          </div>
        </div>

        <div class="path-item">
          <span class="path-label">Cache Directory:</span>
          <div class="path-value">
            <code>{systemPaths.cache_dir}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.cache_dir)}>
              📋
            </button>
          </div>
        </div>

        <div class="path-item">
          <span class="path-label">Temp Directory:</span>
          <div class="path-value">
            <code>{systemPaths.temp_dir}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.temp_dir)}>
              📋
            </button>
          </div>
        </div>

        <div class="path-item">
          <span class="path-label">Current Directory:</span>
          <div class="path-value">
            <code>{systemPaths.current_dir}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.current_dir)}>
              📋
            </button>
          </div>
        </div>

        {#if systemPaths.chrome_path}
          <div class="path-item">
            <span class="path-label">Chrome Path:</span>
            <div class="path-value">
              <code>{systemPaths!.chrome_path}</code>
              <button class="copy-btn" on:click={() => copyToClipboard(systemPaths!.chrome_path!)}>
                📋
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </section>

  <!-- Test Server Section -->
  <section class="section">
    <h3>🌐 Mock Test Server</h3>
    <p class="description">
      Start a local HTTP server for manual e2e testing. Use this to test CDP commands against a
      controlled webpage environment.
    </p>

    <div class="server-status">
      <div class="status-indicator">
        <span class="status-dot {serverStatus.running ? 'running' : 'stopped'}"></span>
        <span class="status-text">
          {serverStatus.running ? 'Running' : 'Stopped'}
        </span>
      </div>

      {#if serverStatus.running && serverStatus.url}
        <div class="server-info">
          <div class="info-item">
            <strong>URL:</strong>
            <a href={serverStatus.url} target="_blank">{serverStatus.url}</a>
            <button class="copy-btn" on:click={() => copyToClipboard(serverStatus.url!)}>
              📋
            </button>
          </div>
          <div class="info-item">
            <strong>Port:</strong>
            {serverStatus.port}
          </div>
        </div>
      {/if}
    </div>

    <div class="server-controls">
      {#if !serverStatus.running}
        <button class="btn btn-primary" on:click={handleStartServer} disabled={loading}>
          {loading ? 'Starting...' : 'Start Server'}
        </button>
      {:else}
        <button class="btn btn-primary" on:click={handleOpenInBrowser} disabled={loading}>
          {loading ? 'Loading...' : 'Open in Browser'}
        </button>
        <button class="btn btn-secondary" on:click={handleStopServer} disabled={loading}>
          {loading ? 'Stopping...' : 'Stop Server'}
        </button>
      {/if}
    </div>

    {#if serverStatus.running}
      <div class="server-guide">
        <h4>💡 Quick Start Guide</h4>
        <ol>
          <li>Click "Open in Browser" to navigate to the test page</li>
          <li>Use the chat interface to test CDP commands</li>
          <li>The test page includes interactive elements for testing automation</li>
          <li>Check the DebugView to see CDP commands being executed</li>
        </ol>
      </div>
    {/if}
  </section>

  <!-- CDP Debugger Section -->
  <section class="section">
    <CdpDebugger />
  </section>

  <!-- Screenshot Management Section -->
  <section class="section">
    <h3>📸 Screenshot Management</h3>
    <p class="description">
      Manually capture screenshots for testing and debugging. Screenshots are saved to the temp
      directory and can be used with Claude API.
    </p>

    <div class="screenshot-controls">
      <button
        class="btn btn-primary"
        on:click={handleCaptureScreenshot}
        disabled={screenshotLoading}
      >
        {screenshotLoading ? 'Capturing...' : '📸 Save Screenshot'}
      </button>
      <button class="btn btn-secondary" on:click={loadScreenshots} disabled={screenshotLoading}>
        🔄 Refresh List
      </button>
      {#if screenshots.length > 0}
        <button
          class="btn btn-danger"
          on:click={handleDeleteAllScreenshots}
          disabled={screenshotLoading}
        >
          {screenshotLoading ? 'Deleting...' : '🗑️ Delete All'}
        </button>
      {/if}
    </div>

    {#if screenshots.length === 0}
      <div class="empty-state">
        No screenshots captured yet. Click "Save Screenshot" to capture the current page.
      </div>
    {:else}
      <div class="screenshot-list">
        <div class="list-header">
          <span class="count"
            >{screenshots.length} screenshot{screenshots.length !== 1 ? 's' : ''}</span
          >
        </div>
        {#each screenshots as screenshot (screenshot.path)}
          <div class="screenshot-item">
            <div class="screenshot-info">
              <div class="screenshot-filename">{screenshot.filename}</div>
              <div class="screenshot-meta">
                <span class="meta-item">📅 {formatTimestamp(screenshot.timestamp)}</span>
                <span class="meta-item">💾 {formatSize(screenshot.size_kb)}</span>
              </div>
            </div>
            <div class="screenshot-actions">
              <button
                class="copy-btn"
                on:click={() => copyToClipboard(screenshot.path)}
                title="Copy path"
              >
                📋
              </button>
              <button
                class="delete-btn"
                on:click={() => handleDeleteScreenshot(screenshot.path)}
                title="Delete"
              >
                🗑️
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .developer-mode {
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  h2 {
    margin: 0 0 1.5rem 0;
    font-size: 1.75rem;
    color: #333;
  }

  .error-banner {
    background: #fee;
    border: 1px solid #fcc;
    color: #c33;
    padding: 1rem;
    margin-bottom: 1rem;
    border-radius: 4px;
  }

  .section {
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  h3 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
    color: #667eea;
  }

  .description {
    color: #666;
    margin-bottom: 1rem;
    line-height: 1.5;
  }

  .loading {
    color: #999;
    font-style: italic;
  }

  .paths-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .path-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .path-label {
    font-weight: 600;
    color: #555;
    font-size: 0.875rem;
  }

  .path-value {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: #f8f8f8;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #e0e0e0;
  }

  .path-value code {
    flex: 1;
    font-size: 0.875rem;
    word-break: break-all;
    color: #333;
  }

  .copy-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    padding: 0.25rem 0.5rem;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .copy-btn:hover {
    opacity: 1;
  }

  .server-status {
    margin-bottom: 1rem;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .status-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    display: inline-block;
  }

  .status-dot.running {
    background: #4caf50;
    box-shadow: 0 0 8px rgba(76, 175, 80, 0.6);
    animation: pulse 2s infinite;
  }

  .status-dot.stopped {
    background: #999;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .status-text {
    font-weight: 600;
    font-size: 1rem;
  }

  .server-info {
    background: #f0f8ff;
    padding: 1rem;
    border-radius: 4px;
    border: 1px solid #d0e8ff;
  }

  .info-item {
    margin-bottom: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .info-item:last-child {
    margin-bottom: 0;
  }

  .info-item a {
    color: #667eea;
    text-decoration: none;
  }

  .info-item a:hover {
    text-decoration: underline;
  }

  .server-controls {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #667eea;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #5568d3;
  }

  .btn-secondary {
    background: #e0e0e0;
    color: #333;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #d0d0d0;
  }

  .server-guide {
    background: #fffbea;
    padding: 1rem;
    border-radius: 4px;
    border: 1px solid #ffe8a1;
  }

  .server-guide h4 {
    margin: 0 0 0.75rem 0;
    color: #976a00;
    font-size: 1rem;
  }

  .server-guide ol {
    margin: 0;
    padding-left: 1.5rem;
    color: #666;
  }

  .server-guide li {
    margin-bottom: 0.5rem;
    line-height: 1.5;
  }

  .server-guide li:last-child {
    margin-bottom: 0;
  }

  /* Screenshot Management Styles */
  .screenshot-controls {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .btn-danger {
    background: #dc3545;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #c82333;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    color: #999;
    font-style: italic;
    background: #f8f8f8;
    border-radius: 4px;
    border: 1px dashed #ddd;
  }

  .screenshot-list {
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    overflow: hidden;
  }

  .list-header {
    background: #f8f8f8;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #e0e0e0;
  }

  .count {
    font-weight: 600;
    color: #555;
    font-size: 0.875rem;
  }

  .screenshot-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #f0f0f0;
    transition: background 0.2s;
  }

  .screenshot-item:hover {
    background: #fafafa;
  }

  .screenshot-item:last-child {
    border-bottom: none;
  }

  .screenshot-info {
    flex: 1;
    min-width: 0;
  }

  .screenshot-filename {
    font-weight: 500;
    color: #333;
    margin-bottom: 0.25rem;
    font-size: 0.9rem;
    word-break: break-all;
  }

  .screenshot-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.8rem;
    color: #666;
  }

  .meta-item {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .screenshot-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: 1rem;
  }

  .delete-btn {
    background: transparent;
    border: 1px solid #ddd;
    cursor: pointer;
    font-size: 1rem;
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .delete-btn:hover {
    background: #fee;
    border-color: #fcc;
  }
</style>
