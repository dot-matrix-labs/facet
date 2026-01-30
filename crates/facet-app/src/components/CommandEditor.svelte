<script lang="ts">
  import { onMount } from 'svelte';
  import { saveCommand, getCommand } from '../lib/tauri';
  import type { Command, CommandParameter, ParameterType } from '../lib/types';

  export let commandName: string | null = null; // null for new command
  export let onSave: () => void;
  export let onCancel: () => void;

  let name = '';
  let description = '';
  let version = '1.0.0';
  let browserProfile = '';
  let parameters: CommandParameter[] = [];
  let rules: string[] = [];
  let checklist: string[] = [];
  let includeCdpScript = false;
  let cdpScriptTemplate = '';
  let loading = false;
  let error = '';
  let isEditing = false;
  let showPreview = false;
  let editorMode: 'wizard' | 'markdown' = 'wizard'; // Toggle between wizard and markdown editor
  let markdownContent = ''; // For direct markdown editing

  // Temporary input fields for adding items
  let newRule = '';
  let newChecklistItem = '';

  onMount(async () => {
    if (commandName) {
      isEditing = true;
      await loadCommand(commandName);
    } else {
      // New command - set default CDP script template example
      cdpScriptTemplate = JSON.stringify(
        {
          name: 'My Command',
          cdp_commands: [
            {
              method: 'Page.navigate',
              params: {
                url: 'https://example.com',
              },
            },
          ],
        },
        null,
        2
      );
    }
  });

  async function loadCommand(cmdName: string) {
    loading = true;
    error = '';
    try {
      const result = await getCommand(cmdName);
      if (result.success && result.data) {
        const cmd = result.data;
        name = cmd.frontmatter.command_name;
        description = cmd.frontmatter.description;
        version = cmd.frontmatter.version;
        browserProfile = cmd.frontmatter.browser_profile || '';
        parameters = cmd.parameters;
        rules = cmd.rules;
        checklist = cmd.checklist;
        if (cmd.cdp_script_template) {
          includeCdpScript = true;
          cdpScriptTemplate = cmd.cdp_script_template;
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

  async function handleSave() {
    error = '';

    // Validation
    if (!name.trim()) {
      error = 'Command name is required';
      return;
    }

    if (!/^[a-z0-9-]+$/.test(name)) {
      error = 'Command name must be kebab-case (lowercase letters, numbers, and dashes only)';
      return;
    }

    if (!description.trim()) {
      error = 'Description is required';
      return;
    }

    if (!version.match(/^\d+\.\d+\.\d+$/)) {
      error = 'Version must be in semantic version format (e.g., 1.0.0)';
      return;
    }

    // Validate CDP script if included
    if (includeCdpScript && cdpScriptTemplate.trim()) {
      try {
        JSON.parse(cdpScriptTemplate);
      } catch {
        error = 'CDP script must be valid JSON';
        return;
      }
    }

    loading = true;

    const command: Command = {
      frontmatter: {
        command_name: name.trim(),
        description: description.trim(),
        browser_profile: browserProfile.trim() || undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        version: version.trim(),
        changelog: [],
      },
      parameters,
      rules,
      checklist,
      generative_ui: undefined,
      cdp_script_template:
        includeCdpScript && cdpScriptTemplate.trim() ? cdpScriptTemplate.trim() : undefined,
    };

    try {
      const result = await saveCommand(command);
      if (result.success) {
        onSave();
      } else {
        error = result.error || 'Failed to save command';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to save command:', e);
    } finally {
      loading = false;
    }
  }

  function addParameter() {
    parameters = [
      ...parameters,
      {
        name: '',
        param_type: { type: 'text_input' },
        label: '',
        placeholder: '',
        required: true,
        default: undefined,
      },
    ];
  }

  function removeParameter(index: number) {
    parameters = parameters.filter((_, i) => i !== index);
  }

  function getParameterTypeLabel(paramType: ParameterType): string {
    if (paramType.type === 'text_input') return 'Text Input';
    if (paramType.type === 'short_text') return 'Short Text';
    if (paramType.type === 'dropdown') return 'Dropdown';
    if (paramType.type === 'radio') return 'Radio';
    if (paramType.type === 'checkbox') return 'Checkbox';
    if (paramType.type === 'slider') return 'Slider';
    if (paramType.type === 'color_picker') return 'Color Picker';
    if (paramType.type === 'date_picker') return 'Date Picker';
    return 'Unknown';
  }

  function updateParameterType(index: number, typeStr: string) {
    const param = parameters[index];
    if (typeStr === 'text_input') {
      param.param_type = { type: 'text_input' };
    } else if (typeStr === 'short_text') {
      param.param_type = { type: 'short_text', max_length: undefined };
    } else if (typeStr === 'dropdown') {
      param.param_type = { type: 'dropdown', options: [] };
    } else if (typeStr === 'radio') {
      param.param_type = { type: 'radio', options: [] };
    } else if (typeStr === 'checkbox') {
      param.param_type = { type: 'checkbox' };
    } else if (typeStr === 'slider') {
      param.param_type = { type: 'slider', min: 0, max: 100, step: 1, unit: undefined };
    } else if (typeStr === 'color_picker') {
      param.param_type = { type: 'color_picker' };
    } else if (typeStr === 'date_picker') {
      param.param_type = { type: 'date_picker' };
    }
    // Trigger reactivity by reassigning array
    parameters = [...parameters];
  }

  function addRule() {
    if (newRule.trim()) {
      rules = [...rules, newRule.trim()];
      newRule = '';
    }
  }

  function removeRule(index: number) {
    rules = rules.filter((_, i) => i !== index);
  }

  function addChecklistItem() {
    if (newChecklistItem.trim()) {
      checklist = [...checklist, newChecklistItem.trim()];
      newChecklistItem = '';
    }
  }

  function removeChecklistItem(index: number) {
    checklist = checklist.filter((_, i) => i !== index);
  }

  function formatCdpScript() {
    try {
      const parsed = JSON.parse(cdpScriptTemplate);
      cdpScriptTemplate = JSON.stringify(parsed, null, 2);
      error = '';
    } catch {
      error = 'Cannot format: CDP script must be valid JSON';
    }
  }

  function generateMarkdownPreview(): string {
    let md = '---\n';
    md += `command_name: ${name || 'untitled'}\n`;
    md += `description: ${description || 'No description'}\n`;
    if (browserProfile) {
      md += `browser_profile: ${browserProfile}\n`;
    }
    md += `version: ${version}\n`;
    md += `created_at: ${new Date().toISOString()}\n`;
    md += `updated_at: ${new Date().toISOString()}\n`;
    md += 'changelog: []\n';
    md += '---\n\n';

    if (parameters.length > 0) {
      md += '## Parameters\n\n';
      parameters.forEach((p) => {
        const req = p.required ? ' (required)' : ' (optional)';
        md += `- **${p.name}** (${getParameterTypeLabel(p.param_type)}${req}): ${p.label}\n`;
      });
      md += '\n';
    }

    if (rules.length > 0) {
      md += '## Rules\n\n';
      rules.forEach((r) => {
        md += `- ${r}\n`;
      });
      md += '\n';
    }

    if (checklist.length > 0) {
      md += '## Checklist\n\n';
      checklist.forEach((c) => {
        md += `- [ ] ${c}\n`;
      });
      md += '\n';
    }

    if (includeCdpScript && cdpScriptTemplate.trim()) {
      md += '## CDP Script Template\n\n';
      md += '```json\n';
      md += cdpScriptTemplate;
      md += '\n```\n';
    }

    return md;
  }

  function switchToMarkdownMode() {
    // Generate markdown from current wizard values
    markdownContent = generateMarkdownPreview();
    editorMode = 'markdown';
  }

  function switchToWizardMode() {
    // Parse markdown back to wizard fields (basic parsing)
    // For now, just switch mode - we can enhance parsing later
    editorMode = 'wizard';
  }
</script>

<div class="command-editor">
  <div class="header">
    <h2>{isEditing ? 'Edit Command' : 'New Command'}</h2>
    <div class="mode-toggle">
      <button
        type="button"
        class="mode-btn"
        class:active={editorMode === 'wizard'}
        on:click={switchToWizardMode}
      >
        Wizard
      </button>
      <button
        type="button"
        class="mode-btn"
        class:active={editorMode === 'markdown'}
        on:click={switchToMarkdownMode}
      >
        Markdown
      </button>
    </div>
    <button class="btn-close" on:click={onCancel}>×</button>
  </div>

  {#if loading && isEditing}
    <div class="loading">Loading command...</div>
  {:else}
    <form on:submit|preventDefault={handleSave}>
      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      {#if editorMode === 'markdown'}
        <!-- Markdown Editor Mode -->
        <div class="section">
          <h3>Markdown Editor</h3>
          <p class="hint">
            Edit the command markdown directly. Switch to Wizard mode for a guided form.
          </p>
          <textarea
            class="markdown-editor"
            bind:value={markdownContent}
            placeholder="Enter your command markdown here..."
            rows="25"
          ></textarea>
          <p class="hint">
            Note: Markdown parsing will be added soon. For now, use Wizard mode to create commands.
          </p>
        </div>
      {:else}
        <!-- Wizard Mode -->
        <!-- Frontmatter Section -->
        <div class="section">
          <h3>Command Information</h3>

          <div class="form-group">
            <label for="name">
              Command Name <span class="required">*</span>
              <span class="hint">(kebab-case: my-command)</span>
            </label>
            <input
              id="name"
              type="text"
              bind:value={name}
              placeholder="my-command"
              disabled={isEditing}
              required
            />
          </div>

          <div class="form-group">
            <label for="description">
              Description <span class="required">*</span>
              <span class="hint">Describe what this command does and what it achieves</span>
            </label>
            <textarea
              id="description"
              bind:value={description}
              placeholder="This command navigates to a website and extracts product information..."
              rows="4"
              required
            ></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="version">
                Version <span class="required">*</span>
              </label>
              <input id="version" type="text" bind:value={version} placeholder="1.0.0" required />
            </div>

            <div class="form-group">
              <label for="browserProfile">Browser Profile</label>
              <input
                id="browserProfile"
                type="text"
                bind:value={browserProfile}
                placeholder="Optional profile name"
              />
            </div>
          </div>
        </div>

        <!-- Parameters Section -->
        <div class="section">
          <h3>
            Parameters
            <button type="button" class="btn-add" on:click={addParameter}>+ Add Parameter</button>
          </h3>

          {#if parameters.length === 0}
            <div class="empty-message">
              No parameters defined. Click "Add Parameter" to create one.
            </div>
          {:else}
            <div class="items-list">
              {#each parameters as param, i (i)}
                <div class="parameter-item">
                  <div class="param-header">
                    <input
                      type="text"
                      bind:value={param.name}
                      placeholder="parameter_name"
                      class="param-name"
                      required
                    />
                    <select
                      value={param.param_type.type}
                      on:change={(e) => updateParameterType(i, e.currentTarget.value)}
                      class="param-type"
                    >
                      <option value="text_input">Text Input</option>
                      <option value="short_text">Short Text</option>
                      <option value="dropdown">Dropdown</option>
                      <option value="radio">Radio</option>
                      <option value="checkbox">Checkbox</option>
                      <option value="slider">Slider</option>
                      <option value="color_picker">Color Picker</option>
                      <option value="date_picker">Date Picker</option>
                    </select>
                    <button
                      type="button"
                      class="btn-remove"
                      on:click={() => removeParameter(i)}
                      title="Remove parameter"
                    >
                      ×
                    </button>
                  </div>
                  <input
                    type="text"
                    bind:value={param.label}
                    placeholder="User-facing label"
                    class="param-label"
                    required
                  />
                  <div class="param-options">
                    <label class="checkbox-label">
                      <input type="checkbox" bind:checked={param.required} />
                      Required
                    </label>
                    <input
                      type="text"
                      bind:value={param.placeholder}
                      placeholder="Placeholder text (optional)"
                      class="param-placeholder"
                    />
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Rules Section -->
        <div class="section">
          <h3>Rules</h3>
          <p class="section-hint">Constraints and guidelines for executing this command</p>

          <div class="add-item">
            <input
              type="text"
              bind:value={newRule}
              placeholder="Add a rule or constraint..."
              on:keydown={(e) => e.key === 'Enter' && (e.preventDefault(), addRule())}
            />
            <button type="button" class="btn-add-small" on:click={addRule}>Add</button>
          </div>

          {#if rules.length > 0}
            <ul class="items-list">
              {#each rules as rule, i (i)}
                <li>
                  <span class="item-text">{rule}</span>
                  <button
                    type="button"
                    class="btn-remove-small"
                    on:click={() => removeRule(i)}
                    title="Remove rule"
                  >
                    ×
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <!-- Checklist Section -->
        <div class="section">
          <h3>Success Checklist</h3>
          <p class="section-hint">Criteria to verify successful execution</p>

          <div class="add-item">
            <input
              type="text"
              bind:value={newChecklistItem}
              placeholder="Add success criterion..."
              on:keydown={(e) => e.key === 'Enter' && (e.preventDefault(), addChecklistItem())}
            />
            <button type="button" class="btn-add-small" on:click={addChecklistItem}>Add</button>
          </div>

          {#if checklist.length > 0}
            <ul class="items-list">
              {#each checklist as item, i (i)}
                <li>
                  <span class="item-text">{item}</span>
                  <button
                    type="button"
                    class="btn-remove-small"
                    on:click={() => removeChecklistItem(i)}
                    title="Remove item"
                  >
                    ×
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <!-- CDP Script Template Section (Optional) -->
        <div class="section">
          <h3>
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={includeCdpScript} />
              Include Static CDP Script Template (Optional)
            </label>
          </h3>
          <p class="section-hint">
            If not provided, AI will generate CDP commands dynamically from the command description
          </p>

          {#if includeCdpScript}
            <div class="form-group">
              <label for="cdpScript">
                CDP Script JSON
                <button type="button" class="btn-format" on:click={formatCdpScript}>
                  Format JSON
                </button>
              </label>
              <textarea
                id="cdpScript"
                bind:value={cdpScriptTemplate}
                placeholder="Enter CDP script JSON"
                rows="12"
              ></textarea>
              <div class="hint">This will be used as fallback if AI generation is unavailable</div>
            </div>
          {/if}
        </div>

        <!-- Preview Section -->
        <div class="section">
          <h3>
            <button type="button" class="btn-preview" on:click={() => (showPreview = !showPreview)}>
              {showPreview ? '▼' : '▶'} Preview Markdown
            </button>
          </h3>

          {#if showPreview}
            <pre class="markdown-preview">{generateMarkdownPreview()}</pre>
          {/if}
        </div>
      {/if}
      <!-- End Wizard Mode -->

      <!-- Form Actions -->
      <div class="form-actions">
        <button type="button" class="btn-cancel" on:click={onCancel} disabled={loading}>
          Cancel
        </button>
        <button type="submit" class="btn-save" disabled={loading}>
          {loading ? 'Saving...' : 'Save Command'}
        </button>
      </div>
    </form>
  {/if}
</div>

<style>
  .command-editor {
    padding: 1.5rem;
    max-width: 900px;
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
    font-size: 1.5rem;
    color: #333;
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

  .error-message {
    padding: 0.75rem;
    background: #fee;
    color: #c33;
    border-radius: 4px;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }

  .section {
    margin-bottom: 2rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid #f0f0f0;
  }

  .section:last-of-type {
    border-bottom: none;
  }

  .section h3 {
    font-size: 1.1rem;
    margin-bottom: 1rem;
    color: #333;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .section-hint {
    color: #999;
    font-size: 0.85rem;
    margin: -0.5rem 0 1rem 0;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
    font-size: 0.95rem;
  }

  .required {
    color: #c33;
  }

  .hint {
    font-weight: normal;
    color: #999;
    font-size: 0.85rem;
    margin-left: 0.5rem;
  }

  input[type='text'],
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

  input:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  .btn-add {
    margin-left: auto;
    padding: 0.25rem 0.75rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-add:hover {
    background: #005a9e;
  }

  .btn-format {
    float: right;
    padding: 0.25rem 0.75rem;
    background: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-format:hover {
    background: #e0e0e0;
  }

  .btn-preview {
    padding: 0.25rem 0.75rem;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .btn-preview:hover {
    background: #f0f0f0;
  }

  .empty-message {
    padding: 1rem;
    background: #f9f9f9;
    border: 1px dashed #ddd;
    border-radius: 4px;
    text-align: center;
    color: #666;
    font-size: 0.9rem;
  }

  .items-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .items-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 4px;
  }

  .item-text {
    flex: 1;
    color: #333;
  }

  .parameter-item {
    padding: 0.75rem;
    background: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .param-header {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .param-name {
    flex: 2;
  }

  .param-type {
    flex: 1;
  }

  .param-label,
  .param-placeholder {
    width: 100%;
  }

  .param-options {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: normal;
    margin-bottom: 0;
  }

  .btn-remove {
    font-size: 1.5rem;
    background: transparent;
    border: none;
    cursor: pointer;
    color: #c33;
    padding: 0;
    width: 2rem;
    height: 2rem;
  }

  .btn-remove:hover {
    color: #a11;
  }

  .add-item {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .add-item input {
    flex: 1;
  }

  .btn-add-small {
    padding: 0.5rem 1rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .btn-add-small:hover {
    background: #005a9e;
  }

  .btn-remove-small {
    padding: 0.25rem 0.5rem;
    background: transparent;
    border: none;
    cursor: pointer;
    color: #c33;
    font-size: 1.2rem;
  }

  .btn-remove-small:hover {
    color: #a11;
  }

  .markdown-preview {
    padding: 1rem;
    background: #f9f9f9;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.85rem;
    overflow-x: auto;
    white-space: pre-wrap;
    max-height: 400px;
    overflow-y: auto;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 2px solid #f0f0f0;
  }

  .btn-cancel,
  .btn-save {
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

  .btn-save {
    background: #007acc;
    color: white;
  }

  .btn-save:hover:not(:disabled) {
    background: #005a9e;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Mode Toggle */
  .mode-toggle {
    display: flex;
    gap: 0.5rem;
    background: #f0f0f0;
    border-radius: 6px;
    padding: 0.25rem;
  }

  .mode-btn {
    padding: 0.5rem 1rem;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    color: #666;
    transition: all 0.2s;
  }

  .mode-btn:hover {
    background: #e0e0e0;
  }

  .mode-btn.active {
    background: white;
    color: #0066cc;
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  /* Markdown Editor */
  .markdown-editor {
    width: 100%;
    min-height: 500px;
    padding: 1rem;
    font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
    font-size: 0.9rem;
    line-height: 1.6;
    border: 1px solid #ddd;
    border-radius: 4px;
    resize: vertical;
  }

  .markdown-editor:focus {
    outline: none;
    border-color: #0066cc;
    box-shadow: 0 0 0 3px rgba(0, 102, 204, 0.1);
  }

  /* Enhanced Description Field */
  textarea#description {
    min-height: 100px;
    resize: vertical;
    line-height: 1.5;
  }
</style>
