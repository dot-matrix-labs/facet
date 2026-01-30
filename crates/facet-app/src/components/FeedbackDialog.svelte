<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import { invoke } from '@tauri-apps/api/core';

  export let show = false;

  const dispatch = createEventDispatcher();

  type Message = {
    role: 'user' | 'assistant';
    content: string;
    isError?: boolean;
  };

  type WorkflowResult = {
    success: boolean;
    workflow_type: string; // 'feedback_improvement'
    message: string;
    refined_feedback?: string | null;
    error?: string;
  };

  let messages: Message[] = [
    {
      role: 'assistant',
      content: "Hi! I'm here to help you submit feedback or report a bug. What's on your mind?",
    },
  ];

  let inputText = '';
  let isLoading = false;
  let refinedFeedback: string | null = null;
  let isSubmitting = false;
  let chatContainer: HTMLDivElement;
  let inputElement: HTMLTextAreaElement;

  // Scroll to bottom when messages change
  $: if (messages) {
    scrollToBottom();
  }

  async function scrollToBottom() {
    await tick();
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleSubmitInput() {
    if (!inputText.trim() || isLoading) return;

    const userInput = inputText.trim();
    inputText = '';

    // Add user message
    messages = [...messages, { role: 'user', content: userInput }];
    isLoading = true;

    try {
      // Call Backend to Improve Feedback
      const result = await invoke<WorkflowResult>('process_chat_message', {
        request: {
          message: userInput,
          workflow_type: 'feedback_improvement',
          agent_name: 'feedback-assistant',
          include_screenshot: false,
          include_html: false,
        },
      });

      if (result.success) {
        messages = [...messages, { role: 'assistant', content: result.message }];

        if (result.refined_feedback) {
          refinedFeedback = result.refined_feedback;
        }
      } else {
        messages = [
          ...messages,
          {
            role: 'assistant',
            content: result.message || 'I encountered an error.',
            isError: true,
          },
        ];
      }
    } catch (err: any) {
      console.error('Feedback error:', err);
      messages = [...messages, { role: 'assistant', content: `Error: ${err}`, isError: true }];
    } finally {
      isLoading = false;
      inputElement?.focus();
    }
  }

  async function handleFinalSubmit() {
    if (!refinedFeedback) return;

    isSubmitting = true;
    try {
      await invoke('submit_application_feedback', {
        feedback: {
          title: 'User Feedback', // We could extract a title, but generic is fine for now
          description: refinedFeedback,
          email: null, // Optional: could grab from user store if logged in
        },
      });

      messages = [
        ...messages,
        {
          role: 'assistant',
          content: '✅ Feedback submitted successfully! Thank you for your help.',
        },
      ];
      refinedFeedback = null; // Hide submit button

      // Auto close after 2 seconds
      setTimeout(() => {
        handleClose();
        // Reset state for next time
        messages = [
          {
            role: 'assistant',
            content:
              "Hi! I'm here to help you submit feedback or report a bug. What's on your mind?",
          },
        ];
      }, 2500);
    } catch (err: any) {
      console.error('Submit error:', err);
      messages = [
        ...messages,
        { role: 'assistant', content: `Failed to submit: ${err}`, isError: true },
      ];
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmitInput();
    }
  }
</script>

{#if show}
  <div class="feedback-overlay" on:click|self={handleClose} transition:fade>
    <div class="feedback-dialog">
      <div class="header">
        <h3>Submit Feedback</h3>
        <button class="close-btn" on:click={handleClose}>&times;</button>
      </div>

      <div class="chat-area" bind:this={chatContainer}>
        {#each messages as msg}
          <div class="message {msg.role} {msg.isError ? 'error' : ''}">
            <div class="avatar">
              {#if msg.role === 'assistant'}
                🤖
              {:else}
                👤
              {/if}
            </div>
            <div class="bubble">
              {msg.content}
            </div>
          </div>
        {/each}

        {#if isLoading}
          <div class="message assistant">
            <div class="avatar">🤖</div>
            <div class="bubble typing">
              <span>.</span><span>.</span><span>.</span>
            </div>
          </div>
        {/if}
      </div>

      {#if refinedFeedback}
        <div class="review-area">
          <div class="review-label">Ready to Submit:</div>
          <div class="review-content">{refinedFeedback}</div>
          <div class="review-actions">
            <button
              class="cancel-btn"
              on:click={() => (refinedFeedback = null)}
              disabled={isSubmitting}>Edit</button
            >
            <button class="submit-btn" on:click={handleFinalSubmit} disabled={isSubmitting}>
              {#if isSubmitting}Submitting...{:else}Submit Feedback{/if}
            </button>
          </div>
        </div>
      {:else}
        <div class="input-area">
          <textarea
            bind:this={inputElement}
            bind:value={inputText}
            on:keydown={handleKeydown}
            placeholder="Describe the issue or idea..."
            rows="2"
            disabled={isLoading}
          ></textarea>
          <button
            class="send-btn"
            on:click={handleSubmitInput}
            disabled={!inputText.trim() || isLoading}
          >
            <svg viewBox="0 0 24 24" fill="none" class="send-icon">
              <path
                d="M22 2L11 13"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M22 2L15 22L11 13L2 9L22 2Z"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .feedback-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.5);
    z-index: 2000;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }

  .feedback-dialog {
    background: white;
    width: 450px;
    height: 600px;
    border-radius: 12px;
    box-shadow:
      0 20px 25px -5px rgba(0, 0, 0, 0.1),
      0 10px 10px -5px rgba(0, 0, 0, 0.04);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: scaleIn 0.2s ease-out;
  }

  @keyframes scaleIn {
    from {
      transform: scale(0.95);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
  }

  .header {
    padding: 1rem;
    border-bottom: 1px solid #e5e7eb;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #f9fafb;
  }

  .header h3 {
    margin: 0;
    font-size: 1.125rem;
    color: #111827;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #6b7280;
    line-height: 1;
  }

  .close-btn:hover {
    color: #111827;
  }

  .chat-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    background: #ffffff;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .message {
    display: flex;
    gap: 0.75rem;
    max-width: 85%;
  }

  .message.user {
    align-self: flex-end;
    flex-direction: row-reverse;
  }

  .avatar {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #f3f4f6;
    border-radius: 50%;
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  .message.user .avatar {
    background: #eef2ff;
  }

  .bubble {
    background: #f3f4f6;
    padding: 0.75rem;
    border-radius: 12px;
    border-top-left-radius: 2px;
    color: #374151;
    font-size: 0.9375rem;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .message.user .bubble {
    background: #4f46e5;
    color: white;
    border-radius: 12px;
    border-top-right-radius: 2px;
    border-top-left-radius: 12px;
  }

  .message.error .bubble {
    background: #fee2e2;
    color: #b91c1c;
    border: 1px solid #fecaca;
  }

  .input-area {
    padding: 1rem;
    border-top: 1px solid #e5e7eb;
    background: #ffffff;
    display: flex;
    gap: 0.75rem;
    align-items: flex-end;
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    resize: none;
    font-family: inherit;
    font-size: 0.9375rem;
    outline: none;
    transition: border-color 0.15s;
  }

  textarea:focus {
    border-color: #4f46e5;
  }

  .send-btn {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    border: none;
    background: #4f46e5;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .send-icon {
    width: 20px;
    height: 20px;
  }

  .review-area {
    padding: 1rem;
    background: #f3f4f6;
    border-top: 1px solid #e5e7eb;
  }

  .review-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: #4b5563;
    margin-bottom: 0.5rem;
  }

  .review-content {
    background: white;
    padding: 0.75rem;
    border-radius: 6px;
    border: 1px solid #e5e7eb;
    margin-bottom: 1rem;
    max-height: 150px;
    overflow-y: auto;
    font-size: 0.9375rem;
    color: #111827;
  }

  .review-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .cancel-btn {
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    background: white;
    color: #374151;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .submit-btn {
    padding: 0.5rem 1rem;
    background: #10b981;
    color: white;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .submit-btn:hover {
    background: #059669;
  }

  .typing span {
    animation: blink 1.4s infinite both;
    font-size: 1.5rem;
    line-height: 1rem;
    margin: 0 1px;
  }

  .typing span:nth-child(2) {
    animation-delay: 0.2s;
  }
  .typing span:nth-child(3) {
    animation-delay: 0.4s;
  }

  @keyframes blink {
    0% {
      opacity: 0.2;
    }
    20% {
      opacity: 1;
    }
    100% {
      opacity: 0.2;
    }
  }
</style>
