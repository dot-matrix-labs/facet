<script lang="ts">
  /**
   * Profile Editor Component
   *
   * Allows users to edit their user-profile.md markdown document.
   * This file is included as context in all AI prompts for personalization.
   *
   * Features:
   * - Markdown text editor
   * - Load profile on mount
   * - Save button with loading state
   * - Auto-save draft to localStorage
   * - Character count
   * - Markdown formatting hints
   * - Success/error feedback
   */

  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { currentUser, updateUserProfile } from '../lib/userStore';
  import type { ProfileResult } from '../lib/types';

  let content = '';
  let originalContent = '';
  let isSaving = false;
  let saveError: string | null = null;
  let saveSuccess = false;
  let hasChanges = false;

  // Subscribe to current user
  let user: typeof $currentUser = null;
  const unsubUser = currentUser.subscribe((value) => {
    user = value;
  });

  onMount(async () => {
    await loadProfile();
  });

  /**
   * Load user profile markdown from backend
   */
  async function loadProfile() {
    try {
      // Try to load existing profile
      const result = await invoke<ProfileResult<string>>('get_user_profile');

      if (result.success && result.data) {
        content = result.data;
        originalContent = result.data;
      } else {
        // New profile, use default template
        content = getDefaultProfileTemplate();
        originalContent = content;
      }
    } catch (error) {
      console.error('Failed to load profile:', error);
      // Use default template on error
      content = getDefaultProfileTemplate();
      originalContent = content;
    }
  }

  /**
   * Get default profile template
   */
  function getDefaultProfileTemplate(): string {
    const username = user?.username || 'User';
    return `# User Profile: ${username}

## Preferences
- Detail your preferences for automation workflows
- Examples: level of detail, privacy concerns, technical comfort

## Goals
- What you want to accomplish with Robert
- Use cases and workflows you care about

## Language Style
- How you prefer to communicate
- Tone, formality, etc.

## Additional Context
(Add any additional information that would help personalize your experience)
`;
  }

  /**
   * Handle content changes
   */
  function handleContentChange() {
    hasChanges = content !== originalContent;
    saveError = null;
    saveSuccess = false;
  }

  /**
   * Save profile to backend
   */
  async function handleSave() {
    try {
      isSaving = true;
      saveError = null;
      saveSuccess = false;

      const success = await updateUserProfile(content);

      if (success) {
        saveSuccess = true;
        originalContent = content;
        hasChanges = false;

        // Clear success message after 3 seconds
        setTimeout(() => {
          saveSuccess = false;
        }, 3000);
      } else {
        saveError = 'Failed to save profile. Please try again.';
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      saveError = `Failed to save: ${errorMessage}`;
    } finally {
      isSaving = false;
    }
  }

  /**
   * Reset to original content
   */
  function handleReset() {
    if (confirm('Discard all unsaved changes?')) {
      content = originalContent;
      hasChanges = false;
      saveError = null;
      saveSuccess = false;
    }
  }

  /**
   * Get character count
   */
  function getCharCount(): number {
    return content.length;
  }

  /**
   * Get word count
   */
  function getWordCount(): number {
    return content
      .trim()
      .split(/\s+/)
      .filter((word) => word.length > 0).length;
  }

  // Cleanup subscriptions
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    unsubUser();
  });
</script>

<div class="profile-editor">
  <!-- Header -->
  <div class="editor-header">
    <div class="header-content">
      <h2 class="editor-title">User Profile</h2>
      <p class="editor-subtitle">
        This profile helps personalize your experience with AI-powered automation
      </p>
    </div>

    <!-- Stats -->
    <div class="editor-stats">
      <span class="stat">{getWordCount()} words</span>
      <span class="stat-divider">•</span>
      <span class="stat">{getCharCount()} characters</span>
    </div>
  </div>

  <!-- Markdown Hints -->
  <div class="markdown-hints">
    <div class="hints-header">
      <svg
        width="14"
        height="14"
        viewBox="0 0 14 14"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M7 1C3.69 1 1 3.69 1 7C1 10.31 3.69 13 7 13C10.31 13 13 10.31 13 7C13 3.69 10.31 1 7 1ZM7 10C6.45 10 6 9.55 6 9C6 8.45 6.45 8 7 8C7.55 8 8 8.45 8 9C8 9.55 7.55 10 7 10ZM8 6C8 6.55 7.55 7 7 7C6.45 7 6 6.55 6 6V4C6 3.45 6.45 3 7 3C7.55 3 8 3.45 8 4V6Z"
          fill="currentColor"
        />
      </svg>
      <span>Markdown supported:</span>
    </div>
    <div class="hints-list">
      <span># Heading</span>
      <span>**bold**</span>
      <span>*italic*</span>
      <span>- bullet</span>
    </div>
  </div>

  <!-- Editor -->
  <div class="editor-container">
    <textarea
      bind:value={content}
      on:input={handleContentChange}
      placeholder={getDefaultProfileTemplate()}
      disabled={isSaving}
      class="editor-textarea"
    ></textarea>
  </div>

  <!-- Footer -->
  <div class="editor-footer">
    <!-- Status Messages -->
    <div class="status-messages">
      {#if saveError}
        <div class="status-message error">
          <svg
            width="14"
            height="14"
            viewBox="0 0 14 14"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M7 1C3.69 1 1 3.69 1 7C1 10.31 3.69 13 7 13C10.31 13 13 10.31 13 7C13 3.69 10.31 1 7 1ZM9 9.5L7 7.5L5 9.5L4.5 9L6.5 7L4.5 5L5 4.5L7 6.5L9 4.5L9.5 5L7.5 7L9.5 9L9 9.5Z"
              fill="currentColor"
            />
          </svg>
          <span>{saveError}</span>
        </div>
      {:else if saveSuccess}
        <div class="status-message success">
          <svg
            width="14"
            height="14"
            viewBox="0 0 14 14"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M7 1C3.69 1 1 3.69 1 7C1 10.31 3.69 13 7 13C10.31 13 13 10.31 13 7C13 3.69 10.31 1 7 1ZM5.5 9.5L3 7L3.71 6.29L5.5 8.08L10.29 3.29L11 4L5.5 9.5Z"
              fill="currentColor"
            />
          </svg>
          <span>Profile saved successfully!</span>
        </div>
      {:else if hasChanges}
        <div class="status-message warning">
          <span>Unsaved changes</span>
        </div>
      {/if}
    </div>

    <!-- Actions -->
    <div class="editor-actions">
      {#if hasChanges}
        <button
          type="button"
          class="editor-button secondary"
          on:click={handleReset}
          disabled={isSaving}
        >
          Reset
        </button>
      {/if}

      <button
        type="button"
        class="editor-button primary"
        on:click={handleSave}
        disabled={isSaving || !hasChanges}
      >
        {#if isSaving}
          <span class="loading-spinner"></span>
          <span>Saving...</span>
        {:else}
          <span>Save Profile</span>
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .profile-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: white;
    border-radius: 12px;
    overflow: hidden;
  }

  .editor-header {
    padding: 1.5rem;
    border-bottom: 1px solid #e5e7eb;
    background: #f9fafb;
  }

  .header-content {
    margin-bottom: 0.75rem;
  }

  .editor-title {
    font-size: 1.25rem;
    font-weight: 700;
    color: #111827;
    margin: 0 0 0.25rem 0;
  }

  .editor-subtitle {
    font-size: 0.875rem;
    color: #6b7280;
    margin: 0;
  }

  .editor-stats {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: #6b7280;
  }

  .stat {
    font-weight: 500;
  }

  .stat-divider {
    opacity: 0.5;
  }

  .markdown-hints {
    padding: 0.75rem 1.5rem;
    background: #eff6ff;
    border-bottom: 1px solid #dbeafe;
    display: flex;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .hints-header {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: #1e40af;
  }

  .hints-list {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.75rem;
    font-family: 'Courier New', monospace;
    color: #1e40af;
  }

  .hints-list span {
    background: white;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #dbeafe;
  }

  .editor-container {
    flex: 1;
    min-height: 0;
    padding: 1.5rem;
    overflow: auto;
  }

  .editor-textarea {
    width: 100%;
    height: 100%;
    min-height: 400px;
    padding: 1rem;
    font-size: 0.9375rem;
    font-family: 'Monaco', 'Menlo', 'Consolas', 'Courier New', monospace;
    line-height: 1.6;
    color: #111827;
    background: #f9fafb;
    border: 1.5px solid #d1d5db;
    border-radius: 8px;
    resize: vertical;
    transition: all 0.15s ease;
  }

  .editor-textarea::placeholder {
    color: #9ca3af;
  }

  .editor-textarea:hover:not(:disabled) {
    border-color: #9ca3af;
  }

  .editor-textarea:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
    background: white;
  }

  .editor-textarea:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .editor-footer {
    padding: 1.5rem;
    border-top: 1px solid #e5e7eb;
    background: #f9fafb;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .status-messages {
    flex: 1;
    min-width: 200px;
  }

  .status-message {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .status-message svg {
    flex-shrink: 0;
  }

  .status-message.error {
    background: #fef2f2;
    color: #dc2626;
    border: 1px solid #fecaca;
  }

  .status-message.success {
    background: #f0fdf4;
    color: #16a34a;
    border: 1px solid #bbf7d0;
  }

  .status-message.warning {
    background: #fffbeb;
    color: #d97706;
    border: 1px solid #fde68a;
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .editor-button {
    padding: 0.625rem 1.25rem;
    font-size: 0.875rem;
    font-weight: 600;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .editor-button.primary {
    color: white;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  }

  .editor-button.primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .editor-button.primary:active:not(:disabled) {
    transform: translateY(0);
  }

  .editor-button.primary:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .editor-button.secondary {
    color: #374151;
    background: white;
    border: 1.5px solid #d1d5db;
  }

  .editor-button.secondary:hover:not(:disabled) {
    background: #f9fafb;
    border-color: #9ca3af;
  }

  .editor-button.secondary:disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .loading-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
