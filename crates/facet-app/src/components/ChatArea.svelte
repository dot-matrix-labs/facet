<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let messages = [{ role: 'assistant', content: 'Hello! I am Robert. How can I help you today?' }];
  let input = '';

  async function sendMessage() {
    if (!input.trim()) return;

    const userMsg = input;
    messages = [...messages, { role: 'user', content: userMsg }];
    input = '';

    try {
      const response = await invoke<string>('chat', { message: userMsg });
      messages = [...messages, { role: 'assistant', content: response }];
    } catch (e) {
      console.error('Chat failed:', e);
      messages = [
        ...messages,
        { role: 'assistant', content: 'Error: Failed to connect to Robert backend.' },
      ];
    }
  }
</script>

<div class="flex-1 flex flex-col h-full bg-gray-950">
  <!-- Chat History -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#each messages as msg}
      <div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'}">
        <div
          class="max-w-2xl px-4 py-2 rounded-lg {msg.role === 'user'
            ? 'bg-indigo-600 text-white'
            : 'bg-gray-800 text-gray-200'}"
        >
          {msg.content}
        </div>
      </div>
    {/each}
  </div>

  <!-- Input Area -->
  <div class="p-4 border-t border-gray-800">
    <div class="flex space-x-4">
      <input
        type="text"
        bind:value={input}
        on:keydown={(e) => e.key === 'Enter' && sendMessage()}
        placeholder="Ask anything..."
        class="flex-1 bg-gray-900 text-white border-gray-700 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
      />
      <button
        on:click={sendMessage}
        class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
      >
        Send
      </button>
    </div>
  </div>
</div>
