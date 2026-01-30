<script lang="ts">
  import { onMount } from 'svelte';
  import { executeCdpScript } from '../lib/tauri';
  import type { ExecutionReport } from '../lib/types';

  // Template CDP script for clicking the "Click Me" button
  const defaultTemplate = `{
  "name": "click-test-button",
  "description": "Demo script that clicks the 'Click Me' button on the test server",
  "author": "You",
  "tags": ["demo", "test", "click"],
  "cdp_commands": [
    {
      "method": "Runtime.evaluate",
      "params": {
        "expression": "document.getElementById('alert-button').click()",
        "returnByValue": true
      },
      "description": "Click the 'Click Me' button by ID"
    },
    {
      "method": "Runtime.evaluate",
      "params": {
        "expression": "document.getElementById('output').innerText",
        "returnByValue": true
      },
      "description": "Read the output text to verify the click worked"
    }
  ]
}`;

  let scriptJson = defaultTemplate;
  let executionReport: ExecutionReport | null = null;
  let executing = false;
  let error: string | null = null;
  let validationError: string | null = null;

  // Validate JSON as user types
  function validateJson() {
    try {
      JSON.parse(scriptJson);
      validationError = null;
    } catch (e) {
      validationError = `Invalid JSON: ${e instanceof Error ? e.message : String(e)}`;
    }
  }

  async function handleExecute() {
    error = null;
    executionReport = null;
    executing = true;

    try {
      // Validate JSON first
      validateJson();
      if (validationError) {
        error = validationError;
        return;
      }

      // Execute the CDP script
      const report = await executeCdpScript(scriptJson);
      executionReport = report;
    } catch (e) {
      error = `Execution failed: ${e}`;
      console.error('CDP execution error:', e);
    } finally {
      executing = false;
    }
  }

  function loadTemplate() {
    scriptJson = defaultTemplate;
    validationError = null;
    error = null;
    executionReport = null;
  }

  function formatDuration(duration: { secs: number; nanos: number }): string {
    const totalMs = duration.secs * 1000 + duration.nanos / 1_000_000;
    if (totalMs < 1000) {
      return `${totalMs.toFixed(0)}ms`;
    }
    return `${(totalMs / 1000).toFixed(2)}s`;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'success':
        return 'status-success';
      case 'failed':
        return 'status-failed';
      case 'skipped':
        return 'status-skipped';
      default:
        return '';
    }
  }

  // Validate on mount
  onMount(() => {
    validateJson();
  });
</script>

<div class="cdp-debugger">
  <div class="header">
    <h3>CDP Script Debugger</h3>
    <p class="description">
      Test and debug CDP (Chrome DevTools Protocol) scripts. Write or edit JSON instructions to
      automate browser interactions.
    </p>
  </div>

  <div class="editor-section">
    <div class="editor-header">
      <label for="cdp-script">CDP Script (JSON)</label>
      <div class="editor-actions">
        <button class="btn btn-small btn-secondary" on:click={loadTemplate}> Load Template </button>
      </div>
    </div>

    <textarea
      id="cdp-script"
      bind:value={scriptJson}
      on:input={validateJson}
      placeholder="Enter CDP script JSON..."
      spellcheck="false"
    ></textarea>

    {#if validationError}
      <div class="validation-error">
        {validationError}
      </div>
    {/if}
  </div>

  <div class="execute-section">
    <button
      class="btn btn-primary btn-large"
      on:click={handleExecute}
      disabled={executing || !!validationError}
    >
      {#if executing}
        Executing...
      {:else}
        Execute CDP Script
      {/if}
    </button>
  </div>

  {#if error}
    <div class="error-banner">
      {error}
    </div>
  {/if}

  {#if executionReport}
    <div class="results-section">
      <h4>Execution Report</h4>

      <div class="report-summary">
        <div class="summary-grid">
          <div class="summary-item">
            <span class="summary-label">Script:</span>
            <span class="summary-value">{executionReport.script_name}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Total Commands:</span>
            <span class="summary-value">{executionReport.total_commands}</span>
          </div>
          <div class="summary-item success">
            <span class="summary-label">Successful:</span>
            <span class="summary-value">{executionReport.successful}</span>
          </div>
          <div class="summary-item failed">
            <span class="summary-label">Failed:</span>
            <span class="summary-value">{executionReport.failed}</span>
          </div>
          <div class="summary-item skipped">
            <span class="summary-label">Skipped:</span>
            <span class="summary-value">{executionReport.skipped}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Duration:</span>
            <span class="summary-value">{formatDuration(executionReport.total_duration)}</span>
          </div>
        </div>
      </div>

      <div class="command-results">
        <h5>Command Results</h5>
        {#each executionReport.results as result}
          <div class="result-item {getStatusColor(result.status)}">
            <div class="result-header">
              <span class="step-number">Step {result.step}</span>
              <span class="method-name">{result.method}</span>
              <span class="status-badge {result.status}">{result.status.toUpperCase()}</span>
              <span class="duration">{formatDuration(result.duration)}</span>
            </div>

            {#if result.error}
              <div class="result-error">
                <strong>Error:</strong>
                {result.error}
              </div>
            {/if}

            {#if result.response}
              <details class="result-details">
                <summary>Response Data</summary>
                <pre class="response-data">{JSON.stringify(result.response, null, 2)}</pre>
              </details>
            {/if}

            {#if result.saved_file}
              <div class="saved-file">
                Saved to: <code>{result.saved_file}</code>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .cdp-debugger {
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
  }

  .header {
    margin-bottom: 1.5rem;
  }

  h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    color: #667eea;
  }

  .description {
    color: #666;
    margin: 0;
    line-height: 1.5;
  }

  .editor-section {
    margin-bottom: 1rem;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  label {
    font-weight: 600;
    color: #555;
    font-size: 0.875rem;
  }

  .editor-actions {
    display: flex;
    gap: 0.5rem;
  }

  textarea {
    width: 100%;
    min-height: 400px;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: 'Courier New', Monaco, monospace;
    font-size: 0.875rem;
    line-height: 1.5;
    resize: vertical;
    background: #f8f8f8;
  }

  textarea:focus {
    outline: none;
    border-color: #667eea;
    background: white;
  }

  .validation-error {
    margin-top: 0.5rem;
    padding: 0.75rem;
    background: #fee;
    border: 1px solid #fcc;
    color: #c33;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .execute-section {
    margin-bottom: 1.5rem;
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

  .btn-small {
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
  }

  .btn-large {
    width: 100%;
    padding: 1rem 2rem;
    font-size: 1.1rem;
  }

  .error-banner {
    background: #fee;
    border: 1px solid #fcc;
    color: #c33;
    padding: 1rem;
    margin-bottom: 1rem;
    border-radius: 4px;
  }

  .results-section {
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  h4 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: #333;
  }

  h5 {
    margin: 1.5rem 0 1rem 0;
    font-size: 1rem;
    color: #555;
  }

  .report-summary {
    background: #f8f8f8;
    padding: 1rem;
    border-radius: 4px;
    border: 1px solid #e0e0e0;
    margin-bottom: 1.5rem;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .summary-label {
    font-size: 0.75rem;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .summary-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: #333;
  }

  .summary-item.success .summary-value {
    color: #4caf50;
  }

  .summary-item.failed .summary-value {
    color: #f44336;
  }

  .summary-item.skipped .summary-value {
    color: #ff9800;
  }

  .command-results {
    margin-top: 1.5rem;
  }

  .result-item {
    background: #fafafa;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
    padding: 1rem;
    margin-bottom: 0.75rem;
  }

  .result-item.status-success {
    border-left: 4px solid #4caf50;
  }

  .result-item.status-failed {
    border-left: 4px solid #f44336;
    background: #fff5f5;
  }

  .result-item.status-skipped {
    border-left: 4px solid #ff9800;
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .step-number {
    font-weight: 600;
    color: #667eea;
    font-size: 0.875rem;
  }

  .method-name {
    flex: 1;
    font-family: 'Courier New', Monaco, monospace;
    font-size: 0.875rem;
    color: #333;
  }

  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 3px;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .status-badge.success {
    background: #e8f5e9;
    color: #2e7d32;
  }

  .status-badge.failed {
    background: #ffebee;
    color: #c62828;
  }

  .status-badge.skipped {
    background: #fff3e0;
    color: #ef6c00;
  }

  .duration {
    font-size: 0.75rem;
    color: #999;
    font-family: 'Courier New', Monaco, monospace;
  }

  .result-error {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: #ffebee;
    border-radius: 4px;
    font-size: 0.875rem;
    color: #c62828;
  }

  .result-details {
    margin-top: 0.75rem;
  }

  .result-details summary {
    cursor: pointer;
    font-weight: 600;
    color: #667eea;
    font-size: 0.875rem;
    padding: 0.5rem;
    background: #f0f4ff;
    border-radius: 4px;
  }

  .result-details summary:hover {
    background: #e8eeff;
  }

  .response-data {
    margin: 0.5rem 0 0 0;
    padding: 1rem;
    background: #2b2b2b;
    color: #f8f8f2;
    border-radius: 4px;
    font-size: 0.8rem;
    overflow-x: auto;
    font-family: 'Courier New', Monaco, monospace;
  }

  .saved-file {
    margin-top: 0.75rem;
    padding: 0.5rem;
    background: #e8f5e9;
    border-radius: 4px;
    font-size: 0.875rem;
    color: #2e7d32;
  }

  .saved-file code {
    font-family: 'Courier New', Monaco, monospace;
    background: rgba(0, 0, 0, 0.05);
    padding: 0.125rem 0.25rem;
    border-radius: 2px;
  }
</style>
