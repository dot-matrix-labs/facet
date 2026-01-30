<script lang="ts">
  /**
   * Login Screen Component
   *
   * Displays profile selector and password input for user authentication.
   * Shown on app launch if users exist in the system.
   *
   * Features:
   * - Profile selector dropdown (populated from availableUsers)
   * - Password input with show/hide toggle
   * - "Create New User" button
   * - Error message display
   * - Loading state during authentication
   * - Enter key support for quick login
   * - Auto-focus on first input
   */

  import { onMount } from 'svelte';
  import {
    availableUsers,
    loginUser,
    isLoading,
    userError,
    clearUserError,
  } from '../lib/userStore';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let selectedUsername = '';
  let password = '';
  let showPassword = false;
  let usernameSelectRef: HTMLSelectElement;

  // Subscribe to store values
  let users: string[] = [];
  let loading = false;
  let error: string | null = null;

  const unsubUsers = availableUsers.subscribe((value) => {
    users = value;
    // Auto-select first user if none selected
    if (users.length > 0 && !selectedUsername) {
      selectedUsername = users[0];
    }
  });

  const unsubLoading = isLoading.subscribe((value) => {
    loading = value;
  });

  const unsubError = userError.subscribe((value) => {
    error = value;
  });

  onMount(() => {
    // Auto-focus username selector
    if (usernameSelectRef) {
      usernameSelectRef.focus();
    }
  });

  /**
   * Handle login submission
   * Validates inputs and calls backend login
   */
  async function handleLogin() {
    // Clear previous errors
    clearUserError();

    // Validate inputs
    if (!selectedUsername) {
      userError.set('Please select a user');
      return;
    }

    if (!password) {
      userError.set('Please enter your password');
      return;
    }

    // Attempt login
    const success = await loginUser(selectedUsername, password);

    if (success) {
      // Login successful, parent component will handle navigation
      dispatch('loginSuccess');

      // Clear password field
      password = '';
    } else {
      // Error message already set by loginUser
      // Clear password field for retry
      password = '';
    }
  }

  /**
   * Handle Enter key in password field
   */
  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !loading) {
      handleLogin();
    }
  }

  /**
   * Toggle password visibility
   */
  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }

  /**
   * Navigate to user creation flow
   */
  function handleCreateNewUser() {
    clearUserError();
    dispatch('createUser');
  }

  // Cleanup subscriptions
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    unsubUsers();
    unsubLoading();
    unsubError();
  });
</script>

<div class="login-container">
  <div class="login-card">
    <!-- Header -->
    <div class="login-header">
      <h1 class="login-title">Welcome to Robert</h1>
      <p class="login-subtitle">Sign in to continue</p>
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

    <!-- Login Form -->
    <form on:submit|preventDefault={handleLogin} class="login-form">
      <!-- Profile Selector -->
      <div class="form-group">
        <label for="username-select" class="form-label">Profile</label>
        <div class="select-wrapper">
          <select
            id="username-select"
            bind:this={usernameSelectRef}
            bind:value={selectedUsername}
            disabled={loading}
            class="form-select"
          >
            {#each users as user}
              <option value={user}>{user}</option>
            {/each}
          </select>
          <svg
            class="select-arrow"
            width="12"
            height="12"
            viewBox="0 0 12 12"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M3 5L6 8L9 5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </div>
      </div>

      <!-- Password Input -->
      <div class="form-group">
        <label for="password-input" class="form-label">Password</label>
        <div class="password-wrapper">
          <input
            id="password-input"
            type={showPassword ? 'text' : 'password'}
            bind:value={password}
            disabled={loading}
            placeholder="Enter your password"
            class="form-input"
            on:keypress={handleKeyPress}
          />
          <button
            type="button"
            class="password-toggle"
            on:click={togglePasswordVisibility}
            disabled={loading}
            title={showPassword ? 'Hide password' : 'Show password'}
          >
            {#if showPassword}
              <!-- Eye Off Icon -->
              <svg
                width="18"
                height="18"
                viewBox="0 0 18 18"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M2 2L16 16M7.5 7.5C7.19 7.81 7 8.23 7 8.7C7 9.66 7.84 10.5 8.8 10.5C9.27 10.5 9.69 10.31 10 10M13.5 13.5C12.35 14.38 10.78 15 9 15C4.5 15 1.5 9 1.5 9C2.24 7.64 3.23 6.5 4.4 5.6L13.5 13.5ZM9.88 5.05C9.59 5.02 9.3 5 9 5C4.5 5 1.5 11 1.5 11C2.06 12.05 2.79 13 3.62 13.85L9.88 5.05ZM11 8.95C11.64 9.59 11.91 10.48 11.65 11.28L11 8.95ZM15.38 12.15L11.57 8.34C11.85 7.54 11.58 6.65 10.94 6.01C10.3 5.37 9.41 5.1 8.61 5.38L6.22 3C7.09 2.68 8.03 2.5 9 2.5C13.5 2.5 16.5 8.5 16.5 8.5C16.01 9.42 15.4 10.26 14.71 11L15.38 12.15Z"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            {:else}
              <!-- Eye Icon -->
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
      </div>

      <!-- Login Button -->
      <button type="submit" disabled={loading} class="login-button">
        {#if loading}
          <span class="loading-spinner"></span>
          <span>Signing in...</span>
        {:else}
          <span>Sign In</span>
        {/if}
      </button>
    </form>

    <!-- Create New User Link -->
    <div class="login-footer">
      <button
        type="button"
        class="create-user-link"
        on:click={handleCreateNewUser}
        disabled={loading}
      >
        Create New User
      </button>
    </div>
  </div>
</div>

<style>
  .login-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 2rem;
  }

  .login-card {
    background: white;
    border-radius: 16px;
    box-shadow:
      0 20px 60px rgba(0, 0, 0, 0.3),
      0 4px 20px rgba(0, 0, 0, 0.2);
    padding: 2.5rem;
    width: 100%;
    max-width: 400px;
  }

  .login-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .login-title {
    font-size: 1.75rem;
    font-weight: 700;
    color: #111827;
    margin: 0 0 0.5rem 0;
  }

  .login-subtitle {
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

  .login-form {
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

  .select-wrapper {
    position: relative;
  }

  .form-select {
    width: 100%;
    padding: 0.75rem 2.5rem 0.75rem 1rem;
    font-size: 1rem;
    color: #111827;
    background: white;
    border: 1.5px solid #d1d5db;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    appearance: none;
  }

  .form-select:hover:not(:disabled) {
    border-color: #9ca3af;
  }

  .form-select:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .form-select:disabled {
    background: #f9fafb;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .select-arrow {
    position: absolute;
    right: 1rem;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
    color: #6b7280;
  }

  .password-wrapper {
    position: relative;
  }

  .form-input {
    width: 100%;
    padding: 0.75rem 3rem 0.75rem 1rem;
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

  .login-button {
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

  .login-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 20px rgba(102, 126, 234, 0.4);
  }

  .login-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .login-button:disabled {
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

  .login-footer {
    margin-top: 1.5rem;
    text-align: center;
  }

  .create-user-link {
    background: none;
    border: none;
    color: #667eea;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0.5rem;
    transition: all 0.15s ease;
  }

  .create-user-link:hover:not(:disabled) {
    color: #764ba2;
    text-decoration: underline;
  }

  .create-user-link:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
