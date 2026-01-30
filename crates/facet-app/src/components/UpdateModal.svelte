<script lang="ts">
  import { onMount } from 'svelte';
  import {
    checkForUpdates,
    downloadAndInstallUpdate,
    formatBytes,
    type UpdateCheckResult,
    type UpdateProgress,
  } from '../lib/updater';

  export let visible = false;
  export let autoCheck = false;

  let updateInfo: UpdateCheckResult | null = null;
  let checking = false;
  let downloading = false;
  let downloadProgress: UpdateProgress | null = null;
  let error: string | null = null;

  async function handleCheckForUpdates() {
    checking = true;
    error = null;

    try {
      updateInfo = await checkForUpdates();

      // If no update info (endpoint error or network issue), silently fail
      if (!updateInfo) {
        console.log('[UpdateModal] No update info available, silently continuing');
        checking = false;

        // Close modal if auto-check failed
        if (autoCheck && visible) {
          visible = false;
        }
        return;
      }

      if (updateInfo && !updateInfo.available) {
        // No update available - only show message if manual check
        if (!autoCheck) {
          // For manual checks, show the "up to date" message
          error = null;
        } else {
          // For auto-checks, don't show anything
          visible = false;
        }
      }
    } catch (err) {
      // Silently log errors, don't show to user
      console.warn('[UpdateModal] Error checking for updates:', err);

      // Close modal on error
      if (visible) {
        visible = false;
      }
    } finally {
      checking = false;
    }
  }

  async function handleDownloadAndInstall() {
    downloading = true;
    error = null;
    downloadProgress = null;

    try {
      await downloadAndInstallUpdate((progress) => {
        downloadProgress = progress;
      });
      // If successful, app will relaunch - no need to update UI
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      error = errorMessage;
      console.error('[UpdateModal] Error downloading/installing update:', err);
      downloading = false;
    }
  }

  function handleClose() {
    if (!downloading) {
      visible = false;
      updateInfo = null;
      error = null;
      downloadProgress = null;
    }
  }

  function handleLater() {
    visible = false;
    updateInfo = null;
    error = null;
  }

  onMount(async () => {
    if (autoCheck) {
      // Wait a few seconds after app startup to check
      await new Promise((resolve) => setTimeout(resolve, 3000));
      await handleCheckForUpdates();

      // Automatically download and install if update is available
      if (updateInfo?.available) {
        console.log('[UpdateModal] Update available, automatically downloading and installing...');
        visible = true; // Show modal to display download progress
        await handleDownloadAndInstall();
        // App will relaunch after successful update
      }
    }
  });

  // Export function to trigger manual check
  export function checkNow() {
    visible = true;
    handleCheckForUpdates();
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-overlay" on:click={handleClose}>
    <div class="modal" on:click|stopPropagation role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h2>
          {#if checking}
            Checking for Updates...
          {:else if updateInfo?.available}
            Update Available
          {:else if updateInfo && !updateInfo.available}
            You're Up to Date
          {:else}
            Software Update
          {/if}
        </h2>
        {#if !downloading}
          <button class="close-btn" on:click={handleClose} aria-label="Close">✕</button>
        {/if}
      </div>

      <div class="modal-body">
        {#if checking}
          <div class="status-message">
            <div class="spinner"></div>
            <p>Checking for updates...</p>
          </div>
        {:else if error}
          <div class="error-message">
            <p>❌ {error}</p>
          </div>
        {:else if downloading}
          <div class="download-section">
            <p>Downloading update...</p>
            {#if downloadProgress}
              <div class="progress-bar">
                <div class="progress-fill" style="width: {downloadProgress.percentage}%"></div>
              </div>
              <p class="progress-text">
                {formatBytes(downloadProgress.downloaded)} / {formatBytes(downloadProgress.total)}
                ({downloadProgress.percentage.toFixed(1)}%)
              </p>
            {:else}
              <div class="progress-bar">
                <div class="progress-fill indeterminate"></div>
              </div>
            {/if}
            <p class="info-text">The app will restart automatically when the update is complete.</p>
          </div>
        {:else if updateInfo?.available}
          <div class="update-info">
            <p class="version-info">
              <strong>Current version:</strong>
              {updateInfo.currentVersion}
              <br />
              <strong>New version:</strong>
              {updateInfo.version}
            </p>

            {#if updateInfo.body}
              <div class="release-notes">
                <h3>Release Notes:</h3>
                <div class="notes-content">
                  {updateInfo.body}
                </div>
              </div>
            {/if}
          </div>
        {:else if updateInfo}
          <div class="status-message">
            <p>✅ You're running the latest version ({updateInfo.currentVersion})</p>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        {#if !checking && !downloading}
          {#if updateInfo?.available && !autoCheck}
            <!-- For manual checks, give user the option to update or postpone -->
            <button class="btn btn-secondary" on:click={handleLater}>Later</button>
            <button class="btn btn-primary" on:click={handleDownloadAndInstall}>
              Update Now
            </button>
          {:else if !updateInfo?.available}
            <!-- Only show close button when no update is available -->
            <button class="btn btn-primary" on:click={handleClose}>Close</button>
          {/if}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal {
    background: white;
    border-radius: 12px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #e5e7eb;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #6b7280;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: #f3f4f6;
    color: #1f2937;
  }

  .modal-body {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .status-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem 0;
  }

  .status-message p {
    margin: 0;
    color: #4b5563;
    font-size: 1rem;
  }

  .error-message {
    padding: 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    color: #991b1b;
  }

  .error-message p {
    margin: 0;
  }

  .update-info {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .version-info {
    padding: 1rem;
    background: #f9fafb;
    border-radius: 8px;
    color: #374151;
    line-height: 1.6;
    margin: 0;
  }

  .release-notes {
    margin-top: 0.5rem;
  }

  .release-notes h3 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: #1f2937;
  }

  .notes-content {
    padding: 1rem;
    background: #f9fafb;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
    color: #4b5563;
    font-size: 0.875rem;
    line-height: 1.5;
  }

  .download-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .download-section p {
    margin: 0;
    color: #4b5563;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
    transition: width 0.3s ease;
    border-radius: 4px;
  }

  .progress-fill.indeterminate {
    width: 30%;
    animation: indeterminate 1.5s ease-in-out infinite;
  }

  @keyframes indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }

  .progress-text {
    text-align: center;
    font-size: 0.875rem;
    font-weight: 500;
    color: #6b7280;
  }

  .info-text {
    text-align: center;
    font-size: 0.875rem;
    color: #6b7280;
    font-style: italic;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid #e5e7eb;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.5rem;
    border-top: 1px solid #e5e7eb;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
    outline: none;
  }

  .btn-primary {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
  }

  .btn-primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .btn-secondary {
    background: white;
    color: #6b7280;
    border: 1px solid #d1d5db;
  }

  .btn-secondary:hover {
    background: #f9fafb;
    border-color: #9ca3af;
  }
</style>
