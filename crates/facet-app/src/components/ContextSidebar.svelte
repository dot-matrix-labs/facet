<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Context {
    id: string;
    name: string;
    description: string;
    icon?: string;
  }

  let contexts: Context[] = [];
  let activeContextId = 'personal';

  function getIcon(ctx: Context): string {
    if (ctx.id === 'personal') return '👤';
    if (ctx.id === 'work') return '💼';
    return '📁';
  }

  onMount(async () => {
    try {
      contexts = await invoke('get_contexts');
    } catch (e) {
      console.error('Failed to fetch contexts:', e);
    }
  });

  function switchContext(id: string) {
    activeContextId = id;
    // TODO: Call backend to switch context
  }
</script>

<div class="w-64 bg-gray-900 h-full flex flex-col border-r border-gray-800">
  <div class="p-4 border-b border-gray-800">
    <h1 class="text-xl font-bold text-white">Robert</h1>
    <span class="text-xs text-gray-500">Alpha v0.5</span>
  </div>

  <div class="flex-1 overflow-y-auto p-2">
    <div class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 px-2">
      Contexts
    </div>
    {#each contexts as ctx}
      <button
        class="w-full flex items-center px-2 py-2 text-sm font-medium rounded-md mb-1 {activeContextId ===
        ctx.id
          ? 'bg-gray-800 text-white'
          : 'text-gray-400 hover:bg-gray-800 hover:text-white'}"
        on:click={() => switchContext(ctx.id)}
      >
        <span class="mr-3">{getIcon(ctx)}</span>
        {ctx.name}
      </button>
    {/each}
  </div>

  <div class="p-4 border-t border-gray-800">
    <button
      class="w-full flex items-center justify-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
    >
      New Context
    </button>
  </div>
</div>
