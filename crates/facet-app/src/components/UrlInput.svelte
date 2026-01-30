<script lang="ts">
  import { onMount } from 'svelte';
  import { launchBrowser, navigateToUrl, getBrowserStatus } from '$lib/tauri';
  import { browserLaunched, isNavigating, currentUrl, currentTitle } from '$lib/stores';

  let urlInput = '';
  let errorMessage = '';
  let isVisible = false;

  onMount(async () => {
    try {
      const status = await getBrowserStatus();
      isVisible = status.is_available;
    } catch (e) {
      console.error('Failed to get browser status:', e);
      isVisible = false;
    }
  });

  async function handleLaunchBrowser() {
    errorMessage = '';
    try {
      await launchBrowser();
      $browserLaunched = true;
    } catch (err) {
      errorMessage = `Failed to launch browser: ${err}`;
      console.error(err);
    }
  }

  async function handleNavigate() {
    if (!urlInput.trim()) {
      errorMessage = 'Please enter a URL';
      return;
    }

    // Ensure browser is launched first
    if (!$browserLaunched) {
      try {
        await handleLaunchBrowser();
      } catch {
        return; // Error already handled
      }
    }

    errorMessage = '';
    $isNavigating = true;

    try {
      const result = await navigateToUrl(urlInput);
      $currentUrl = result.url;
      $currentTitle = result.title;
    } catch (err) {
      const errorStr = String(err);

      // Check if browser connection was lost
      if (errorStr.includes('Browser connection lost')) {
        $browserLaunched = false;
        errorMessage = 'Browser connection lost. Please launch the browser again.';
      } else {
        errorMessage = `Navigation failed: ${err}`;
      }
      console.error(err);
    } finally {
      $isNavigating = false;
    }
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleNavigate();
    }
  }
</script>

{#if isVisible}
  <div class="url-input-container">
    <div class="input-row">
      {#if !$browserLaunched}
        <button on:click={handleLaunchBrowser} class="launch-button"> Launch Browser </button>
      {/if}
      <input
        type="text"
        bind:value={urlInput}
        on:keypress={handleKeyPress}
        placeholder="Enter URL (e.g., https://example.com)"
        disabled={$isNavigating}
        class="url-input"
      />
      <button
        on:click={handleNavigate}
        disabled={$isNavigating || !urlInput.trim()}
        class="go-button"
      >
        {$isNavigating ? 'Loading...' : 'Go'}
      </button>
    </div>

    {#if errorMessage}
      <div class="error-message">{errorMessage}</div>
    {/if}

    {#if $currentTitle}
      <div class="current-page">
        <strong>{$currentTitle}</strong>
        <span class="url-display">{$currentUrl}</span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .url-input-container {
    padding: 1.5rem;
    padding-right: 4rem;
    background: #f5f5f5;
    border-bottom: 2px solid #e0e0e0;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .url-input {
    flex: 1;
    padding: 0.75rem 1rem;
    font-size: 1rem;
    border: 2px solid #d0d0d0;
    border-radius: 6px;
    outline: none;
    transition: border-color 0.2s;
  }

  .url-input:focus {
    border-color: #4a90e2;
  }

  .url-input:disabled {
    background: #e8e8e8;
    cursor: not-allowed;
  }

  .launch-button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: white;
    background: #28a745;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
    white-space: nowrap;
  }

  .launch-button:hover {
    background: #218838;
  }

  .go-button {
    padding: 0.75rem 2rem;
    font-size: 1rem;
    font-weight: 600;
    color: white;
    background: #4a90e2;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .go-button:hover:not(:disabled) {
    background: #357abd;
  }

  .go-button:disabled {
    background: #a0a0a0;
    cursor: not-allowed;
  }

  .error-message {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 4px;
    color: #c33;
  }

  .current-page {
    margin-top: 0.75rem;
    padding: 0.5rem;
    background: #e8f4f8;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .url-display {
    display: block;
    color: #666;
    margin-top: 0.25rem;
    font-size: 0.85rem;
  }
</style>
