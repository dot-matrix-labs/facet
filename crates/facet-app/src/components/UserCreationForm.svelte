<script lang="ts">
  /**
   * User Creation Form Component
   *
   * Displays form for creating a new user account.
   * Shown on first launch (no users) or when user clicks "Create New User".
   *
   * Features:
   * - Username input with real-time validation
   * - Password input with strength indicator
   * - Confirm password field
   * - Create button with loading state
   * - Validation feedback (errors and suggestions)
   * - Back button to return to login (if users exist)
   * - Enter key support
   * - Auto-focus on username input
   */

  import { onMount } from 'svelte';
  import {
    createUser,
    isLoading,
    userError,
    clearUserError,
    validateUsername,
    validatePassword,
  } from '../lib/userStore';
  import type { PasswordValidation } from '../lib/types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let showBackButton = true; // Hide on first launch

  let username = '';
  let password = '';
  let confirmPassword = '';
  let showPassword = false;
  let showConfirmPassword = false;
  let usernameInputRef: HTMLInputElement;

  // Validation states
  let usernameValidation: { valid: boolean; errors: string[] } | null = null;
  let passwordValidation: PasswordValidation | null = null;
  let passwordsMatch = true;

  // Subscribe to store values
  let loading = false;
  let error: string | null = null;

  const unsubLoading = isLoading.subscribe((value) => {
    loading = value;
  });

  const unsubError = userError.subscribe((value) => {
    error = value;
  });

  onMount(() => {
    // Auto-focus username input
    if (usernameInputRef) {
      usernameInputRef.focus();
    }
  });

  /**
   * Validate username as user types
   */
  function handleUsernameInput() {
    if (username.length > 0) {
      usernameValidation = validateUsername(username);
    } else {
      usernameValidation = null;
    }
    clearUserError();
  }

  /**
   * Validate password as user types
   */
  function handlePasswordInput() {
    if (password.length > 0) {
      passwordValidation = validatePassword(password);
    } else {
      passwordValidation = null;
    }

    // Check if passwords match
    if (confirmPassword.length > 0) {
      passwordsMatch = password === confirmPassword;
    }
    clearUserError();
  }

  /**
   * Validate confirm password as user types
   */
  function handleConfirmPasswordInput() {
    if (confirmPassword.length > 0) {
      passwordsMatch = password === confirmPassword;
    } else {
      passwordsMatch = true;
    }
    clearUserError();
  }

  /**
   * Handle form submission
   */
  async function handleCreateUser() {
    // Clear previous errors
    clearUserError();

    // Validate all fields
    const usernameCheck = validateUsername(username);
    const passwordCheck = validatePassword(password);

    usernameValidation = usernameCheck;
    passwordValidation = passwordCheck;

    // Check if passwords match
    if (password !== confirmPassword) {
      passwordsMatch = false;
      userError.set('Passwords do not match');
      return;
    }

    // Stop if validation failed
    if (!usernameCheck.valid || !passwordCheck.valid) {
      if (!usernameCheck.valid) {
        userError.set(usernameCheck.errors[0]);
      } else if (!passwordCheck.valid) {
        userError.set(passwordCheck.errors[0]);
      }
      return;
    }

    // Attempt to create user
    const success = await createUser(username, password);

    if (success) {
      // User created and logged in successfully
      dispatch('userCreated');

      // Clear form
      username = '';
      password = '';
      confirmPassword = '';
      usernameValidation = null;
      passwordValidation = null;
    } else {
      // Error message already set by createUser
    }
  }

  /**
   * Handle Enter key in form fields
   */
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !loading) {
      handleCreateUser();
    }
  }

  /**
   * Toggle password visibility
   */
  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  /**
   * Toggle confirm password visibility
   */
  function toggleConfirmPasswordVisibility() {
    showConfirmPassword = !showConfirmPassword;
  }

  /**
   * Go back to login screen
   */
  function handleBack() {
    clearUserError();
    dispatch('back');
  }

  /**
   * Get password strength color
   */
  function getStrengthColor(strength: 'weak' | 'medium' | 'strong'): string {
    switch (strength) {
      case 'weak':
        return '#dc2626';
      case 'medium':
        return '#f59e0b';
      case 'strong':
        return '#10b981';
    }
  }

  /**
   * Get password strength text
   */
  function getStrengthText(strength: 'weak' | 'medium' | 'strong'): string {
    switch (strength) {
      case 'weak':
        return 'Weak';
      case 'medium':
        return 'Medium';
      case 'strong':
        return 'Strong';
    }
  }

  // Cleanup subscriptions
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    unsubLoading();
    unsubError();
  });
</script>

<div class="create-user-container">
  <div class="create-user-card">
    <!-- Header -->
    <div class="create-user-header">
      <h1 class="create-user-title">Create Your Profile</h1>
      <p class="create-user-subtitle">Set up your account to get started</p>
    </div>

    <!-- Error Message -->
    {#if error}
      <div class="error-message">
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M8 1C4.13 1 1 4.13 1 8C1 11.87 4.13 15 8 15C11.87 15 15 11.87 15 8C15 4.13 11.87 1 8 1ZM8 11C7.45 11 7 10.55 7 10C7 9.45 7.45 9 8 9C8.55 9 9 9.45 9 10C9 10.55 8.55 11 8 11ZM9 7C9 7.55 8.55 8 8 8C7.45 8 7 7.55 7 7V5C7 4.45 7.45 4 8 4C8.55 4 9 4.45 9 5V7Z"
            fill="currentColor"
          />
        </svg>
        <span>{error}</span>
      </div>
    {/if}

    <!-- Create User Form -->
    <form on:submit|preventDefault={handleCreateUser} class="create-user-form">
      <!-- Username Input -->
      <div class="form-group">
        <label for="username-input" class="form-label">Username</label>
        <input
          id="username-input"
          type="text"
          bind:this={usernameInputRef}
          bind:value={username}
          on:input={handleUsernameInput}
          on:keypress={handleKeyPress}
          disabled={loading}
          placeholder="Enter a username"
          class="form-input"
          class:error={usernameValidation && !usernameValidation.valid}
          class:success={usernameValidation && usernameValidation.valid}
        />
        {#if usernameValidation}
          {#if usernameValidation.valid}
            <div class="validation-message success">
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
              <span>Username is valid</span>
            </div>
          {:else}
            <div class="validation-message error">
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
              <span>{usernameValidation.errors[0]}</span>
            </div>
          {/if}
        {/if}
        <p class="form-hint">Alphanumeric characters, underscore, and dash only (max 32 chars)</p>
      </div>

      <!-- Password Input -->
      <div class="form-group">
        <label for="password-input" class="form-label">Password</label>
        <div class="password-wrapper">
          <input
            id="password-input"
            type={showPassword ? 'text' : 'password'}
            bind:value={password}
            on:input={handlePasswordInput}
            on:keypress={handleKeyPress}
            disabled={loading}
            placeholder="Enter a secure password"
            class="form-input"
            class:error={passwordValidation && !passwordValidation.valid}
          />
          <button
            type="button"
            class="password-toggle"
            on:click={togglePasswordVisibility}
            disabled={loading}
            title={showPassword ? 'Hide password' : 'Show password'}
          >
            {#if showPassword}
              <svg
                width="18"
                height="18"
                viewBox="0 0 18 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2 2L16 16M7.5 7.5C7.19 7.81 7 8.23 7 8.7C7 9.66 7.84 10.5 8.8 10.5C9.27 10.5 9.69 10.31 10 10"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {:else}
              <svg
                width="18"
                height="18"
                viewBox="0 0 18 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M1.5 9C1.5 9 4.5 3 9 3C13.5 3 16.5 9 16.5 9C16.5 9 13.5 15 9 15C4.5 15 1.5 9 1.5 9Z"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
                <circle cx="9" cy="9" r="2.5" stroke="currentColor" stroke-width="1.5" />
              </svg>
            {/if}
          </button>
        </div>

        <!-- Password Strength Indicator -->
        {#if passwordValidation && password.length > 0}
          <div class="password-strength">
            <div class="strength-label">
              <span>Strength:</span>
              <span
                class="strength-value"
                style="color: {getStrengthColor(passwordValidation.strength)}"
              >
                {getStrengthText(passwordValidation.strength)}
              </span>
            </div>
            <div class="strength-bar">
              <div
                class="strength-fill"
                style="width: {passwordValidation.strength === 'weak'
                  ? '33%'
                  : passwordValidation.strength === 'medium'
                    ? '66%'
                    : '100%'}; background: {getStrengthColor(passwordValidation.strength)}"
              ></div>
            </div>
          </div>

          <!-- Password Suggestions -->
          {#if passwordValidation.suggestions.length > 0}
            <div class="password-suggestions">
              <p class="suggestions-title">Suggestions:</p>
              <ul class="suggestions-list">
                {#each passwordValidation.suggestions as suggestion}
                  <li>{suggestion}</li>
                {/each}
              </ul>
            </div>
          {/if}
        {/if}

        <p class="form-hint">Minimum 12 characters required</p>
      </div>

      <!-- Confirm Password Input -->
      <div class="form-group">
        <label for="confirm-password-input" class="form-label">Confirm Password</label>
        <div class="password-wrapper">
          <input
            id="confirm-password-input"
            type={showConfirmPassword ? 'text' : 'password'}
            bind:value={confirmPassword}
            on:input={handleConfirmPasswordInput}
            on:keypress={handleKeyPress}
            disabled={loading}
            placeholder="Re-enter your password"
            class="form-input"
            class:error={!passwordsMatch && confirmPassword.length > 0}
            class:success={passwordsMatch && confirmPassword.length > 0 && password.length > 0}
          />
          <button
            type="button"
            class="password-toggle"
            on:click={toggleConfirmPasswordVisibility}
            disabled={loading}
            title={showConfirmPassword ? 'Hide password' : 'Show password'}
          >
            {#if showConfirmPassword}
              <svg
                width="18"
                height="18"
                viewBox="0 0 18 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2 2L16 16M7.5 7.5C7.19 7.81 7 8.23 7 8.7C7 9.66 7.84 10.5 8.8 10.5C9.27 10.5 9.69 10.31 10 10"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {:else}
              <svg
                width="18"
                height="18"
                viewBox="0 0 18 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M1.5 9C1.5 9 4.5 3 9 3C13.5 3 16.5 9 16.5 9C16.5 9 13.5 15 9 15C4.5 15 1.5 9 1.5 9Z"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
                <circle cx="9" cy="9" r="2.5" stroke="currentColor" stroke-width="1.5" />
              </svg>
            {/if}
          </button>
        </div>

        {#if !passwordsMatch && confirmPassword.length > 0}
          <div class="validation-message error">
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
            <span>Passwords do not match</span>
          </div>
        {:else if passwordsMatch && confirmPassword.length > 0 && password.length > 0}
          <div class="validation-message success">
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
            <span>Passwords match</span>
          </div>
        {/if}
      </div>

      <!-- Create Button -->
      <button type="submit" disabled={loading} class="create-button">
        {#if loading}
          <span class="loading-spinner"></span>
          <span>Creating account...</span>
        {:else}
          <span>Create Account</span>
        {/if}
      </button>
    </form>

    <!-- Back Button -->
    {#if showBackButton}
      <div class="create-user-footer">
        <button type="button" class="back-link" on:click={handleBack} disabled={loading}>
          ← Back to Sign In
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .create-user-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 2rem;
    overflow-y: auto;
  }

  .create-user-card {
    background: white;
    border-radius: 16px;
    box-shadow:
      0 20px 60px rgba(0, 0, 0, 0.3),
      0 4px 20px rgba(0, 0, 0, 0.2);
    padding: 2.5rem;
    width: 100%;
    max-width: 480px;
    margin: 2rem 0;
  }

  .create-user-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .create-user-title {
    font-size: 1.75rem;
    font-weight: 700;
    color: #111827;
    margin: 0 0 0.5rem 0;
  }

  .create-user-subtitle {
    font-size: 0.875rem;
    color: #6b7280;
    margin: 0;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    color: #dc2626;
    font-size: 0.875rem;
    margin-bottom: 1.5rem;
  }

  .error-message svg {
    flex-shrink: 0;
  }

  .create-user-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .form-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
  }

  .form-input {
    width: 100%;
    padding: 0.75rem 1rem;
    font-size: 1rem;
    color: #111827;
    background: white;
    border: 1.5px solid #d1d5db;
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .form-input::placeholder {
    color: #9ca3af;
  }

  .form-input:hover:not(:disabled) {
    border-color: #9ca3af;
  }

  .form-input:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .form-input:disabled {
    background: #f9fafb;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .form-input.error {
    border-color: #dc2626;
  }

  .form-input.error:focus {
    border-color: #dc2626;
    box-shadow: 0 0 0 3px rgba(220, 38, 38, 0.1);
  }

  .form-input.success {
    border-color: #10b981;
  }

  .form-input.success:focus {
    border-color: #10b981;
    box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.1);
  }

  .password-wrapper {
    position: relative;
  }

  .password-wrapper .form-input {
    padding-right: 3rem;
  }

  .password-toggle {
    position: absolute;
    right: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: #6b7280;
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .password-toggle:hover:not(:disabled) {
    color: #374151;
    background: #f3f4f6;
  }

  .password-toggle:disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }

  .form-hint {
    font-size: 0.75rem;
    color: #6b7280;
    margin: 0;
  }

  .validation-message {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    margin-top: 0.25rem;
  }

  .validation-message svg {
    flex-shrink: 0;
  }

  .validation-message.error {
    color: #dc2626;
  }

  .validation-message.success {
    color: #10b981;
  }

  .password-strength {
    margin-top: 0.75rem;
  }

  .strength-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.75rem;
    margin-bottom: 0.5rem;
    color: #6b7280;
  }

  .strength-value {
    font-weight: 600;
  }

  .strength-bar {
    height: 4px;
    background: #e5e7eb;
    border-radius: 2px;
    overflow: hidden;
  }

  .strength-fill {
    height: 100%;
    transition:
      width 0.3s ease,
      background 0.3s ease;
  }

  .password-suggestions {
    margin-top: 0.75rem;
    padding: 0.75rem;
    background: #eff6ff;
    border: 1px solid #dbeafe;
    border-radius: 6px;
  }

  .suggestions-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: #1e40af;
    margin: 0 0 0.5rem 0;
  }

  .suggestions-list {
    margin: 0;
    padding-left: 1.25rem;
    font-size: 0.75rem;
    color: #1e40af;
  }

  .suggestions-list li {
    margin: 0.25rem 0;
  }

  .create-button {
    width: 100%;
    padding: 0.875rem 1.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: white;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }

  .create-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
  }

  .create-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .create-button:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }

  .loading-spinner {
    width: 16px;
    height: 16px;
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

  .create-user-footer {
    margin-top: 1.5rem;
    text-align: center;
  }

  .back-link {
    background: none;
    border: none;
    color: #667eea;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0.5rem;
    transition: all 0.15s ease;
  }

  .back-link:hover:not(:disabled) {
    color: #764ba2;
    text-decoration: underline;
  }

  .back-link:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
