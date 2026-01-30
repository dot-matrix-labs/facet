<script lang="ts">
  import { onMount } from 'svelte';
  import { listCommands, deleteCommand } from '../lib/tauri';
  import type { CommandInfo } from '../lib/types';

  // Props for parent component callbacks
  // eslint-disable-next-line no-unused-vars
  export let onEdit: (name: string) => void;
  // eslint-disable-next-line no-unused-vars
  export let onExecute: (name: string) => void;
  export let onCreate: () => void;

  let commands: CommandInfo[] = [];
  let loading = true;
  let error = '';

  onMount(async () => {
    await loadCommands();
  });

  async function loadCommands() {
    loading = true;
    error = '';
    try {
      const result = await listCommands();
      if (result.success && result.data) {
        commands = result.data;
      } else {
        error = result.error || 'Failed to load commands';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to load commands:', e);
    } finally {
      loading = false;
    }
  }

  async function handleDelete(name: string) {
    if (!confirm(`Delete command "${name}"?`)) {
      return;
    }

    try {
      const result = await deleteCommand(name);
      if (result.success) {
        await loadCommands(); // Reload list
      } else {
        error = result.error || 'Failed to delete command';
      }
    } catch (e) {
      error = `Error: ${e}`;
      console.error('Failed to delete command:', e);
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }
</script>

<div class="command-list">
  <div class="header">
    <h2>Saved Commands</h2>
    <button class="btn-create" on:click={onCreate}>+ New Command</button>
  </div>

  {#if loading}
    <div class="loading">Loading commands...</div>
  {:else if error}
    <div class="error">
      {error}
      <button on:click={loadCommands}>Retry</button>
    </div>
  {:else if commands.length === 0}
    <div class="empty">
      <p>No commands saved yet.</p>
      <button on:click={onCreate}>Create your first command</button>
    </div>
  {:else}
    <div class="commands">
      {#each commands as cmd (cmd.command_name)}
        <div class="command-card">
          <div class="command-header">
            <div class="command-title">
              <h3>{cmd.command_name}</h3>
              <span class="version-badge">v{cmd.version}</span>
              {#if cmd.browser_profile}
                <span class="profile-badge" title="Browser Profile">
                  {cmd.browser_profile}
                </span>
              {/if}
            </div>
            <div class="command-actions">
              <button
                class="btn-icon btn-execute"
                on:click={() => onExecute(cmd.command_name)}
                title="Execute"
              >
                ▶
              </button>
              <button
                class="btn-icon btn-edit"
                on:click={() => onEdit(cmd.command_name)}
                title="Edit"
              >
                ✏
              </button>
              <button
                class="btn-icon btn-delete"
                on:click={() => handleDelete(cmd.command_name)}
                title="Delete"
              >
                🗑
              </button>
            </div>
          </div>
          <p class="command-description">{cmd.description}</p>
          <div class="command-meta">
            <span class="meta-item" title="Last updated">
              Updated: {formatDate(cmd.updated_at)}
            </span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .command-list {
    padding: 1rem;
    max-width: 800px;
    margin: 0 auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .header h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .btn-create {
    padding: 0.5rem 1rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .btn-create:hover {
    background: #005a9e;
  }

  .loading,
  .error,
  .empty {
    padding: 2rem;
    text-align: center;
    background: #f5f5f5;
    border-radius: 4px;
    color: #666;
  }

  .error {
    background: #fee;
    color: #c33;
  }

  .error button {
    margin-top: 0.5rem;
    padding: 0.25rem 0.75rem;
    background: #c33;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .empty button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .commands {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .command-card {
    background: white;
    border: 1px solid #ddd;
    border-radius: 6px;
    padding: 1rem;
    transition: box-shadow 0.2s;
  }

  .command-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .command-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.5rem;
  }

  .command-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .command-header h3 {
    margin: 0;
    font-size: 1.1rem;
    color: #333;
    font-family: 'Monaco', 'Courier New', monospace;
  }

  .version-badge {
    padding: 0.15rem 0.5rem;
    background: #e8f4fd;
    color: #007acc;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: 'Monaco', 'Courier New', monospace;
  }

  .profile-badge {
    padding: 0.15rem 0.5rem;
    background: #fff4e6;
    color: #e67e22;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .command-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-icon {
    padding: 0.4rem 0.6rem;
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .btn-icon:hover {
    background: #f0f0f0;
  }

  .btn-execute:hover {
    background: #e8f9ed;
    border-color: #28a745;
    color: #28a745;
  }

  .btn-edit:hover {
    background: #e8f4fd;
    border-color: #007acc;
    color: #007acc;
  }

  .btn-delete:hover {
    background: #fee;
    border-color: #dc3545;
    color: #dc3545;
  }

  .command-description {
    margin: 0.5rem 0;
    color: #666;
    font-size: 0.9rem;
    line-height: 1.4;
  }

  .command-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.8rem;
    color: #999;
    margin-top: 0.5rem;
    padding-top: 0.5rem;
    border-top: 1px solid #f0f0f0;
  }

  .meta-item {
    display: flex;
    align-items: center;
  }
</style>
