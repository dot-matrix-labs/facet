<script lang="ts">
  import { onMount } from 'svelte';
  import { getCommand, buildCommandPrompt, getStaticCdp, executeCdpScript } from '../lib/tauri';
  import type { Command, ExecutionReport, JsonValue, ParameterType } from '../lib/types';

  export let commandName: string;
  export let onClose: () => void;

  let command: Command | null = null;
  let parameterValues: Record<string, JsonValue> = {};
  let loading = false;
  let executing = false;
  let error = '';
  let executionReport: ExecutionReport | null = null;
  let aiPrompt = '';
  let showPrompt = false;
  let generatedCdp = '';
  let showCdp = false;
  let executionMode: 'ai' | 'static' = 'ai';

  onMount(async () => {
    await loadCommand();
  });

  async function loadCommand() {
    loading = true;
    error = '';
    try {
      const result = await getCommand(commandName);
      if (result.success && result.data) {
        command = result.data;
        // Initialize parameter values with defaults
        command.parameters.forEach((param) => {
          if (param.default !== undefined && param.default !== null) {
            parameterValues[param.name] = param.default;
          } else {
            parameterValues[param.name] = getDefaultValueForType(param.param_type);
          }
        });
        // Set execution mode based on whether static CDP exists
        if (command.cdp_script_template) {
          executionMode = 'static';
        }
      } else {
        error = result.error || 'Failed to load command';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to load command:', e);
    } finally {
      loading = false;
    }
  }

  function getDefaultValueForType(paramType: ParameterType): JsonValue {
    if (paramType.type === 'checkbox') return false;
    if (paramType.type === 'slider') return paramType.min || 0;
    if (paramType.type === 'dropdown' && paramType.options.length > 0) return paramType.options[0];
    if (paramType.type === 'radio' && paramType.options.length > 0) return paramType.options[0];
    return '';
  }

  function validateParameters(): string | null {
    if (!command) return 'Command not loaded';

    for (const param of command.parameters) {
      const value = parameterValues[param.name];

      // Check required parameters
      if (param.required) {
        if (value === undefined || value === null || value === '') {
          return `Parameter "${param.label}" is required`;
        }
      }

      // Type-specific validation
      if (value !== '' && value !== null && value !== undefined) {
        if (param.param_type.type === 'slider') {
          const numVal = Number(value);
          if (isNaN(numVal)) {
            return `Parameter "${param.label}" must be a number`;
          }
          if (numVal < param.param_type.min || numVal > param.param_type.max) {
            return `Parameter "${param.label}" must be between ${param.param_type.min} and ${param.param_type.max}`;
          }
        }
      }
    }

    return null;
  }

  async function handleGeneratePrompt() {
    if (!command) return;

    const validationError = validateParameters();
    if (validationError) {
      error = validationError;
      return;
    }

    error = '';
    loading = true;

    try {
      const result = await buildCommandPrompt(commandName, parameterValues);
      if (result.success && result.data) {
        aiPrompt = result.data;
        showPrompt = true;
      } else {
        error = result.error || 'Failed to build AI prompt';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to build prompt:', e);
    } finally {
      loading = false;
    }
  }

  async function handleGetStaticCdp() {
    if (!command) return;

    const validationError = validateParameters();
    if (validationError) {
      error = validationError;
      return;
    }

    error = '';
    loading = true;

    try {
      const result = await getStaticCdp(commandName, parameterValues);
      if (result.success) {
        if (result.data) {
          generatedCdp = result.data;
          showCdp = true;
        } else {
          error = 'No static CDP script available for this command';
        }
      } else {
        error = result.error || 'Failed to get static CDP';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to get static CDP:', e);
    } finally {
      loading = false;
    }
  }

  async function handleExecute() {
    if (!command) return;

    const validationError = validateParameters();
    if (validationError) {
      error = validationError;
      return;
    }

    error = '';
    executionReport = null;
    executing = true;

    try {
      let cdpScript: string | null = null;

      if (executionMode === 'static') {
        // Use static CDP script
        const result = await getStaticCdp(commandName, parameterValues);
        if (result.success && result.data) {
          cdpScript = result.data;
        } else {
          error = 'Static CDP script not available';
          return;
        }
      } else {
        // AI mode - user must paste CDP JSON
        if (!generatedCdp.trim()) {
          error = 'Please generate and paste CDP JSON from AI before executing';
          return;
        }
        cdpScript = generatedCdp;
      }

      if (!cdpScript) {
        error = 'No CDP script to execute';
        return;
      }

      // Validate CDP script is valid JSON
      try {
        JSON.parse(cdpScript);
      } catch {
        error = 'CDP script is not valid JSON';
        return;
      }

      // Execute the CDP script
      const report = await executeCdpScript(cdpScript);
      executionReport = report;

      if (report.failed > 0) {
        error = `Command executed but ${report.failed} step(s) failed`;
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to execute command:', e);
    } finally {
      executing = false;
    }
  }

  function formatDuration(duration: { secs: number; nanos: number }): string {
    const ms = duration.secs * 1000 + duration.nanos / 1000000;
    if (ms < 1000) {
      return `${ms.toFixed(0)}ms`;
    }
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(
      () => {
        // Could add a toast notification here
        console.log('Copied to clipboard');
      },
      (err) => {
        console.error('Failed to copy:', err);
      }
    );
  }
</script>

<div class="command-executor">
  <div class="header">
    <h2>Execute: {commandName}</h2>
    <button class="btn-close" on:click={onClose}>×</button>
  </div>

  {#if loading && !command}
    <div class="loading">Loading command...</div>
  {:else if command}
    <div class="command-info">
      <p class="description">{command.frontmatter.description}</p>
      <div class="info-badges">
        <span class="badge">v{command.frontmatter.version}</span>
        {#if command.frontmatter.browser_profile}
          <span class="badge profile">{command.frontmatter.browser_profile}</span>
        {/if}
        {#if command.cdp_script_template}
          <span class="badge static">Static CDP Available</span>
        {/if}
      </div>
    </div>

    <form on:submit|preventDefault={handleExecute}>
      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <!-- Parameters Section -->
      {#if command.parameters.length > 0}
        <div class="section">
          <h3>Parameters</h3>
          <div class="parameters">
            {#each command.parameters as param}
              <div class="param-group">
                <label for={param.name}>
                  {param.label}
                  {#if param.required}
                    <span class="required">*</span>
                  {/if}
                </label>

                {#if param.param_type.type === 'text_input'}
                  <textarea
                    id={param.name}
                    bind:value={parameterValues[param.name]}
                    placeholder={param.placeholder || 'Enter text'}
                    rows="3"
                    required={param.required}
                  ></textarea>
                {:else if param.param_type.type === 'short_text'}
                  <input
                    id={param.name}
                    type="text"
                    bind:value={parameterValues[param.name]}
                    placeholder={param.placeholder || 'Enter text'}
                    maxlength={param.param_type.max_length}
                    required={param.required}
                  />
                {:else if param.param_type.type === 'dropdown'}
                  <select
                    id={param.name}
                    bind:value={parameterValues[param.name]}
                    required={param.required}
                  >
                    <option value="">-- Select --</option>
                    {#each param.param_type.options as option}
                      <option value={option}>{option}</option>
                    {/each}
                  </select>
                {:else if param.param_type.type === 'radio'}
                  <div class="radio-group">
                    {#each param.param_type.options as option}
                      <label class="radio-label">
                        <input
                          type="radio"
                          name={param.name}
                          value={option}
                          bind:group={parameterValues[param.name]}
                          required={param.required}
                        />
                        {option}
                      </label>
                    {/each}
                  </div>
                {:else if param.param_type.type === 'checkbox'}
                  <label class="checkbox-label">
                    <input
                      id={param.name}
                      type="checkbox"
                      checked={!!parameterValues[param.name]}
                      on:change={(e) => (parameterValues[param.name] = e.currentTarget.checked)}
                    />
                    Enable
                  </label>
                {:else if param.param_type.type === 'slider'}
                  <div class="slider-group">
                    <input
                      id={param.name}
                      type="range"
                      min={param.param_type.min}
                      max={param.param_type.max}
                      step={param.param_type.step}
                      bind:value={parameterValues[param.name]}
                    />
                    <span class="slider-value">
                      {parameterValues[param.name]}
                      {param.param_type.unit || ''}
                    </span>
                  </div>
                {:else if param.param_type.type === 'color_picker'}
                  <input id={param.name} type="color" bind:value={parameterValues[param.name]} />
                {:else if param.param_type.type === 'date_picker'}
                  <input
                    id={param.name}
                    type="date"
                    bind:value={parameterValues[param.name]}
                    required={param.required}
                  />
                {:else}
                  <input
                    id={param.name}
                    type="text"
                    bind:value={parameterValues[param.name]}
                    placeholder={param.placeholder || 'Enter value'}
                    required={param.required}
                  />
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="no-params">This command has no parameters.</div>
      {/if}

      <!-- Execution Mode Selection -->
      <div class="section">
        <h3>Execution Mode</h3>
        <div class="mode-selection">
          <label class="radio-label">
            <input type="radio" bind:group={executionMode} value="ai" />
            <div class="mode-info">
              <strong>Dynamic CDP Generation (AI)</strong>
              <span>Generate CDP commands using AI based on command description</span>
            </div>
          </label>
          {#if command.cdp_script_template}
            <label class="radio-label">
              <input type="radio" bind:group={executionMode} value="static" />
              <div class="mode-info">
                <strong>Static CDP Script</strong>
                <span>Use pre-defined CDP script from command template</span>
              </div>
            </label>
          {:else}
            <div class="mode-disabled">
              <span>Static CDP Script (not available for this command)</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- AI Prompt Generation (Dynamic Mode) -->
      {#if executionMode === 'ai'}
        <div class="section">
          <h3>AI Prompt</h3>
          <p class="section-hint">
            Generate a prompt to send to your AI service (Claude, OpenAI, etc.)
          </p>
          <button
            type="button"
            class="btn-secondary"
            on:click={handleGeneratePrompt}
            disabled={loading}
          >
            {loading ? 'Generating...' : 'Generate AI Prompt'}
          </button>

          {#if showPrompt && aiPrompt}
            <div class="prompt-display">
              <div class="prompt-header">
                <span>AI Prompt (copy and send to Claude/OpenAI)</span>
                <button
                  type="button"
                  class="btn-copy"
                  on:click={() => copyToClipboard(aiPrompt)}
                  title="Copy to clipboard"
                >
                  Copy
                </button>
              </div>
              <pre class="prompt-text">{aiPrompt}</pre>
            </div>
          {/if}

          {#if showPrompt}
            <div class="cdp-input">
              <label for="generatedCdp">
                Paste CDP JSON Response from AI
                <span class="required">*</span>
              </label>
              <textarea
                id="generatedCdp"
                bind:value={generatedCdp}
                placeholder="Paste the CDP JSON generated by AI here..."
                rows="10"
              ></textarea>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Static CDP Display (Static Mode) -->
      {#if executionMode === 'static'}
        <div class="section">
          <h3>Static CDP Script</h3>
          <p class="section-hint">Preview the CDP script that will be executed</p>
          <button
            type="button"
            class="btn-secondary"
            on:click={handleGetStaticCdp}
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Show CDP Script'}
          </button>

          {#if showCdp && generatedCdp}
            <div class="cdp-display">
              <pre class="cdp-text">{generatedCdp}</pre>
            </div>
          {/if}
        </div>
      {/if}

      <!-- Execution Actions -->
      <div class="actions">
        <button type="button" class="btn-cancel" on:click={onClose} disabled={executing}>
          Cancel
        </button>
        <button type="submit" class="btn-execute" disabled={executing || loading}>
          {executing ? 'Executing...' : '▶ Execute Command'}
        </button>
      </div>
    </form>

    <!-- Execution Report -->
    {#if executionReport}
      <div class="execution-report">
        <h3>Execution Report</h3>

        <div class="report-summary">
          <div class="summary-item">
            <span class="label">Status:</span>
            <span
              class="value"
              class:success={executionReport.failed === 0}
              class:failure={executionReport.failed > 0}
            >
              {executionReport.failed === 0 ? 'Success' : 'Failed'}
            </span>
          </div>
          <div class="summary-item">
            <span class="label">Duration:</span>
            <span class="value">{formatDuration(executionReport.total_duration)}</span>
          </div>
          <div class="summary-item">
            <span class="label">Steps:</span>
            <span class="value">
              {executionReport.successful} success, {executionReport.failed} failed, {executionReport.skipped}
              skipped
            </span>
          </div>
        </div>

        <div class="report-details">
          <h4>Command Results:</h4>
          {#each executionReport.results as result}
            <div
              class="result-item"
              class:success={result.status === 'success'}
              class:failed={result.status === 'failed'}
            >
              <div class="result-header">
                <span class="step-num">Step {result.step}:</span>
                <span class="method">{result.method}</span>
                <span class="status">{result.status}</span>
                <span class="duration">{formatDuration(result.duration)}</span>
              </div>
              {#if result.error}
                <div class="result-error">{result.error}</div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="error-message">{error || 'Command not found'}</div>
  {/if}
</div>

<style>
  .command-executor {
    padding: 1.5rem;
    max-width: 800px;
    margin: 0 auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    border-bottom: 2px solid #f0f0f0;
    padding-bottom: 0.75rem;
  }

  .header h2 {
    margin: 0;
    font-size: 1.4rem;
    color: #333;
    font-family: 'Monaco', 'Courier New', monospace;
  }

  .btn-close {
    font-size: 2rem;
    background: transparent;
    border: none;
    cursor: pointer;
    color: #999;
    padding: 0;
    width: 2rem;
    height: 2rem;
    line-height: 1;
  }

  .btn-close:hover {
    color: #333;
  }

  .loading {
    padding: 2rem;
    text-align: center;
    color: #666;
  }

  .command-info {
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #f9f9f9;
    border-radius: 4px;
  }

  .description {
    color: #333;
    font-size: 0.95rem;
    margin: 0 0 0.75rem 0;
  }

  .info-badges {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    background: #e8f4fd;
    color: #007acc;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .badge.profile {
    background: #fff4e6;
    color: #e67e22;
  }

  .badge.static {
    background: #e8f9ed;
    color: #28a745;
  }

  .error-message {
    padding: 0.75rem;
    background: #fee;
    color: #c33;
    border-radius: 4px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }

  .section {
    margin-bottom: 1.5rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid #f0f0f0;
  }

  .section h3 {
    font-size: 1.1rem;
    margin-bottom: 0.75rem;
    color: #333;
  }

  .section-hint {
    color: #999;
    font-size: 0.85rem;
    margin: -0.5rem 0 1rem 0;
  }

  .parameters {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .param-group {
    display: flex;
    flex-direction: column;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
    font-size: 0.9rem;
  }

  .required {
    color: #c33;
  }

  input[type='text'],
  input[type='date'],
  input[type='color'],
  textarea,
  select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: inherit;
  }

  textarea {
    font-family: 'Monaco', 'Courier New', monospace;
    resize: vertical;
  }

  input:focus,
  textarea:focus,
  select:focus {
    outline: none;
    border-color: #007acc;
  }

  .radio-group,
  .checkbox-label {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .radio-label:hover {
    background: #f9f9f9;
  }

  .slider-group {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .slider-group input[type='range'] {
    flex: 1;
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-weight: 600;
    color: #333;
  }

  .no-params {
    padding: 1rem;
    background: #f9f9f9;
    border: 1px dashed #ddd;
    border-radius: 4px;
    text-align: center;
    color: #666;
    margin-bottom: 1.5rem;
  }

  .mode-selection {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .mode-info {
    display: flex;
    flex-direction: column;
    margin-left: 0.5rem;
  }

  .mode-info strong {
    color: #333;
    font-size: 0.95rem;
  }

  .mode-info span {
    color: #666;
    font-size: 0.85rem;
  }

  .mode-disabled {
    padding: 0.75rem;
    background: #f9f9f9;
    border: 1px dashed #ddd;
    border-radius: 4px;
    color: #999;
    font-size: 0.9rem;
  }

  .btn-secondary {
    padding: 0.5rem 1rem;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #5a6268;
  }

  .prompt-display {
    margin-top: 1rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    overflow: hidden;
  }

  .prompt-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: #f9f9f9;
    border-bottom: 1px solid #ddd;
    font-size: 0.85rem;
    font-weight: 500;
    color: #333;
  }

  .btn-copy {
    padding: 0.25rem 0.75rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-copy:hover {
    background: #005a9e;
  }

  .prompt-text {
    padding: 1rem;
    background: #f9f9f9;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.85rem;
    overflow-x: auto;
    white-space: pre-wrap;
    max-height: 300px;
    overflow-y: auto;
    margin: 0;
  }

  .cdp-input {
    margin-top: 1rem;
  }

  .cdp-display {
    margin-top: 1rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    overflow: hidden;
  }

  .cdp-text {
    padding: 1rem;
    background: #f9f9f9;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.85rem;
    overflow-x: auto;
    white-space: pre-wrap;
    max-height: 400px;
    overflow-y: auto;
    margin: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 2px solid #f0f0f0;
  }

  .btn-cancel,
  .btn-execute {
    padding: 0.6rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
  }

  .btn-cancel {
    background: #f0f0f0;
    color: #666;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #e0e0e0;
  }

  .btn-execute {
    background: #28a745;
    color: white;
  }

  .btn-execute:hover:not(:disabled) {
    background: #218838;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .execution-report {
    margin-top: 1.5rem;
    padding-top: 1.5rem;
    border-top: 2px solid #f0f0f0;
  }

  .execution-report h3 {
    font-size: 1.2rem;
    margin-bottom: 1rem;
    color: #333;
  }

  .report-summary {
    background: #f9f9f9;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .summary-item {
    display: flex;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
  }

  .summary-item:last-child {
    margin-bottom: 0;
  }

  .summary-item .label {
    font-weight: 500;
    width: 100px;
    color: #666;
  }

  .summary-item .value {
    color: #333;
  }

  .summary-item .value.success {
    color: #28a745;
    font-weight: 600;
  }

  .summary-item .value.failure {
    color: #dc3545;
    font-weight: 600;
  }

  .report-details h4 {
    font-size: 1rem;
    margin-bottom: 0.75rem;
    color: #333;
  }

  .result-item {
    padding: 0.75rem;
    border-left: 3px solid #ddd;
    margin-bottom: 0.5rem;
    background: #f9f9f9;
    border-radius: 0 4px 4px 0;
  }

  .result-item.success {
    border-left-color: #28a745;
    background: #f8fff9;
  }

  .result-item.failed {
    border-left-color: #dc3545;
    background: #fff8f8;
  }

  .result-header {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    font-size: 0.85rem;
  }

  .step-num {
    font-weight: 600;
    color: #666;
  }

  .method {
    font-family: 'Monaco', 'Courier New', monospace;
    color: #333;
    flex: 1;
  }

  .status {
    padding: 0.2rem 0.5rem;
    border-radius: 3px;
    font-size: 0.8rem;
    font-weight: 500;
    text-transform: uppercase;
  }

  .result-item.success .status {
    background: #d4edda;
    color: #155724;
  }

  .result-item.failed .status {
    background: #f8d7da;
    color: #721c24;
  }

  .duration {
    color: #999;
    font-size: 0.8rem;
  }

  .result-error {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: #fff;
    border-radius: 3px;
    color: #dc3545;
    font-size: 0.85rem;
    font-family: 'Monaco', 'Courier New', monospace;
  }
</style>
