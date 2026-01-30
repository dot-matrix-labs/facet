<script lang="ts">
  import CommandList from './CommandList.svelte';
  import CommandEditor from './CommandEditor.svelte';
  import CommandExecutor from './CommandExecutor.svelte';

  type ViewState =
    | { type: 'list' }
    | { type: 'create' }
    | { type: 'edit'; name: string }
    | { type: 'execute'; name: string };

  let view: ViewState = { type: 'list' };

  function showList() {
    view = { type: 'list' };
  }

  function showCreate() {
    view = { type: 'create' };
  }

  function showEdit(name: string) {
    view = { type: 'edit', name };
  }

  function showExecute(name: string) {
    view = { type: 'execute', name };
  }
</script>

<div class="command-manager">
  {#if view.type === 'list'}
    <CommandList onCreate={showCreate} onEdit={showEdit} onExecute={showExecute} />
  {:else if view.type === 'create'}
    <CommandEditor commandName={null} onSave={showList} onCancel={showList} />
  {:else if view.type === 'edit'}
    <CommandEditor commandName={view.name} onSave={showList} onCancel={showList} />
  {:else if view.type === 'execute'}
    <CommandExecutor commandName={view.name} onClose={showList} />
  {/if}
</div>

<style>
  .command-manager {
    height: 100%;
    overflow-y: auto;
  }
</style>
