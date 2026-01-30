<script lang="ts">
  import { onMount } from 'svelte';
  import { processChatMessage } from '../lib/tauri';
  import type { WorkflowType, ClarificationQuestion } from '../lib/types';
  import UrlInput from './UrlInput.svelte';

  type MessageContent = {
    role: 'user' | 'agent';
    text: string;
    timestamp: Date;
    clarification?: ClarificationQuestion[];
    understanding?: string;
  };

  let message = '';
  let messages: MessageContent[] = [];
  let loading = false;
  let browserLaunched = false;
  let pendingClarification: { questions: ClarificationQuestion[]; originalMessage: string } | null =
    null;
  let clarificationAnswers: Record<number, string> = {};

  onMount(async () => {
    // Check if browser is already launched
    // TODO: Add a command to check browser status
  });

  async function handleSend() {
    if (!message.trim() || loading) return;

    const userMessage = message.trim();
    message = '';

    // Add user message to chat
    messages = [...messages, { role: 'user', text: userMessage, timestamp: new Date() }];

    loading = true;

    try {
      // Ensure browser is launched
      // Browser launching is now handled by the server/agent workflow automatically
      if (!browserLaunched) {
        browserLaunched = true;
      }

      // Call the agent workflow to process the message
      const result = await processChatMessage({
        message: userMessage,
        workflow_type: 'cdp_automation' as WorkflowType,
        agent_name: 'cdp-generator',
        include_screenshot: true,
        include_html: true,
      });

      // Check if clarification is needed
      if (result.clarification && result.clarification.length > 0) {
        // Agent needs clarification
        pendingClarification = {
          questions: result.clarification,
          originalMessage: userMessage,
        };
        clarificationAnswers = {}; // Reset answers

        messages = [
          ...messages,
          {
            role: 'agent',
            text: result.understanding || result.message,
            timestamp: new Date(),
            clarification: result.clarification,
            understanding: result.understanding,
          },
        ];
      } else {
        // Normal response
        messages = [
          ...messages,
          {
            role: 'agent',
            text: result.message,
            timestamp: new Date(),
          },
        ];

        // If there was an error, show it
        if (!result.success && result.error) {
          messages = [
            ...messages,
            {
              role: 'agent',
              text: `Error details: ${result.error}`,
              timestamp: new Date(),
            },
          ];
        }
      }
    } catch (error) {
      console.error('Error processing message:', error);
      messages = [
        ...messages,
        {
          role: 'agent',
          text: `Error: ${error}`,
          timestamp: new Date(),
        },
      ];
    } finally {
      loading = false;
    }
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  function formatTime(date: Date): string {
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  async function handleSubmitClarification() {
    if (!pendingClarification) return;

    // Check if all questions are answered
    const unansweredQuestions = pendingClarification.questions
      .map((_, i) => i)
      .filter((i) => !clarificationAnswers[i]);

    if (unansweredQuestions.length > 0) {
      alert('Please answer all questions before submitting.');
      return;
    }

    loading = true;

    // Build clarification text
    const clarificationText = pendingClarification.questions
      .map((q, i) => `${q.question} → ${clarificationAnswers[i]}`)
      .join('\n');

    // Add clarification as user message
    messages = [
      ...messages,
      {
        role: 'user',
        text: `Clarification:\n${clarificationText}`,
        timestamp: new Date(),
      },
    ];

    const originalMessage = pendingClarification.originalMessage;
    pendingClarification = null;

    try {
      // TODO: We need to send the clarification along with the original message
      // For now, we'll send a combined message
      const combinedMessage = `${originalMessage}\n\nClarification provided:\n${clarificationText}`;

      const result = await processChatMessage({
        message: combinedMessage,
        workflow_type: 'cdp_automation' as WorkflowType,
        agent_name: 'cdp-generator',
        include_screenshot: true,
        include_html: true,
      });

      messages = [
        ...messages,
        {
          role: 'agent',
          text: result.message,
          timestamp: new Date(),
        },
      ];

      if (!result.success && result.error) {
        messages = [
          ...messages,
          {
            role: 'agent',
            text: `Error details: ${result.error}`,
            timestamp: new Date(),
          },
        ];
      }
    } catch (error) {
      console.error('Error processing clarification:', error);
      messages = [
        ...messages,
        {
          role: 'agent',
          text: `Error: ${error}`,
          timestamp: new Date(),
        },
      ];
    } finally {
      loading = false;
    }
  }
</script>

<div class="chat-container">
  <UrlInput />

  <div class="messages-container">
    {#if messages.length === 0}
      <div class="empty-state">
        <div class="welcome-icon">🤖</div>
        <h3>Welcome to Robert!</h3>
        <p>I can help you automate browser tasks.</p>
        <p class="examples-title">Try asking me to:</p>
        <ul class="examples">
          <li>"Click the submit button"</li>
          <li>"Fill in the form with my name"</li>
          <li>"Take a screenshot of the page"</li>
          <li>"Scroll to the bottom"</li>
        </ul>
      </div>
    {:else}
      {#each messages as msg (msg.timestamp.getTime())}
        <div class="message {msg.role}">
          <div class="message-header">
            <span class="role-label">
              {msg.role === 'user' ? 'You' : 'Robert'}
            </span>
            <span class="timestamp">{formatTime(msg.timestamp)}</span>
          </div>
          <div class="message-text">{msg.text}</div>

          {#if msg.clarification && msg.clarification.length > 0}
            <div class="clarification-section">
              <p class="clarification-prompt">I need some clarification:</p>
              {#each msg.clarification as question, i}
                <div class="clarification-question">
                  <label class="question-label" for="clarification-{i}">
                    {question.question}
                  </label>
                  {#if question.context}
                    <p class="question-context">{question.context}</p>
                  {/if}
                  <select
                    id="clarification-{i}"
                    bind:value={clarificationAnswers[i]}
                    class="answer-select"
                    disabled={!pendingClarification}
                  >
                    <option value="">-- Select an option --</option>
                    {#each question.options as option}
                      <option value={option}>{option}</option>
                    {/each}
                  </select>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/each}

      {#if pendingClarification}
        <div class="clarification-submit">
          <button
            on:click={handleSubmitClarification}
            class="submit-clarification-button"
            disabled={loading}
          >
            Submit Answers
          </button>
        </div>
      {/if}
    {/if}
  </div>

  <div class="input-container">
    <textarea
      bind:value={message}
      on:keypress={handleKeyPress}
      placeholder="Type your message... (Shift+Enter for new line)"
      disabled={loading}
      rows="3"
    ></textarea>
    <button on:click={handleSend} disabled={!message.trim() || loading} class="send-button">
      {#if loading}
        <span class="spinner"></span> Thinking...
      {:else}
        Send
      {/if}
    </button>
  </div>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #ffffff;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem;
    background: #f8f9fa;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    color: #666;
    padding: 1rem;
  }

  .welcome-icon {
    font-size: 2.5rem;
    margin-bottom: 0.75rem;
  }

  .empty-state h3 {
    margin: 0 0 0.4rem 0;
    font-size: 1.2rem;
    color: #333;
  }

  .empty-state p {
    margin: 0.2rem 0;
    color: #666;
    font-size: 0.9rem;
  }

  .examples-title {
    margin-top: 1rem !important;
    font-weight: 600;
    color: #667eea;
    font-size: 0.9rem;
  }

  .examples {
    list-style: none;
    padding: 0;
    margin: 0.5rem 0 0 0;
    text-align: left;
    max-width: 400px;
  }

  .examples li {
    margin: 0.4rem 0;
    padding: 0.6rem 0.8rem;
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    color: #555;
    font-family: 'Consolas', 'Monaco', monospace;
    font-size: 0.85rem;
  }

  .message {
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    border-radius: 10px;
    max-width: 85%;
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message.user {
    background: #667eea;
    color: white;
    margin-left: auto;
  }

  .message.agent {
    background: white;
    border: 1px solid #e0e0e0;
    color: #333;
    margin-right: auto;
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.4rem;
    font-size: 0.75rem;
  }

  .role-label {
    font-weight: 600;
    font-size: 0.8rem;
  }

  .message.user .role-label {
    color: rgba(255, 255, 255, 0.9);
  }

  .message.agent .role-label {
    color: #667eea;
  }

  .timestamp {
    opacity: 0.7;
    font-size: 0.75rem;
  }

  .message-text {
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
  }

  .clarification-section {
    margin-top: 1rem;
    padding: 1rem;
    background: #f8f9fa;
    border-radius: 8px;
    border: 1px solid #dee2e6;
  }

  .clarification-prompt {
    font-weight: 600;
    color: #495057;
    margin: 0 0 1rem 0;
  }

  .clarification-question {
    margin-bottom: 1rem;
  }

  .clarification-question:last-child {
    margin-bottom: 0;
  }

  .question-label {
    display: block;
    font-weight: 500;
    color: #212529;
    margin-bottom: 0.5rem;
  }

  .question-context {
    font-size: 0.85rem;
    color: #6c757d;
    font-style: italic;
    margin: 0.25rem 0 0.5rem 0;
  }

  .answer-select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ced4da;
    border-radius: 4px;
    font-size: 0.95rem;
    background: white;
    cursor: pointer;
  }

  .answer-select:disabled {
    background: #e9ecef;
    cursor: not-allowed;
  }

  .answer-select:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.1);
  }

  .clarification-submit {
    display: flex;
    justify-content: center;
    padding: 1rem;
    background: #f8f9fa;
    border-top: 2px solid #dee2e6;
    margin-top: 1rem;
  }

  .submit-clarification-button {
    padding: 0.75rem 2rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .submit-clarification-button:hover:not(:disabled) {
    background: #5568d3;
  }

  .submit-clarification-button:disabled {
    background: #adb5bd;
    cursor: not-allowed;
  }

  .input-container {
    padding: 0.75rem;
    border-top: 1px solid #e0e0e0;
    background: white;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  textarea {
    width: 100%;
    padding: 0.6rem;
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    font-family: inherit;
    font-size: 0.9rem;
    resize: none;
    transition: border-color 0.2s;
  }

  textarea:focus {
    outline: none;
    border-color: #667eea;
  }

  textarea:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  .send-button {
    align-self: flex-end;
    padding: 0.6rem 1.5rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .send-button:hover:not(:disabled) {
    background: #5568d3;
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  }

  .send-button:disabled {
    background: #ccc;
    cursor: not-allowed;
    transform: none;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
