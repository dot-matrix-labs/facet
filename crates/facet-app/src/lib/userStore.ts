/**
 * User Profile State Management
 *
 * This module manages user authentication state and provides reactive stores
 * for the current user session. It integrates with the Rust backend's user
 * management system via Tauri commands.
 *
 * Architecture:
 * - Uses Svelte stores for reactive state management
 * - Calls Tauri commands for all backend operations
 * - Automatically checks for users on app start
 * - Persists current user info in memory (cleared on logout)
 *
 * Security Notes:
 * - Never stores passwords in frontend
 * - All sensitive data encrypted by backend
 * - User session cleared on logout or app close
 */

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { UserConfig, ProfileResult, PasswordValidation } from './types';

/**
 * Current user configuration (null if not logged in)
 */
export const currentUser = writable<UserConfig | null>(null);

/**
 * Loading state for async operations
 */
export const isLoading = writable<boolean>(false);

/**
 * Error message for user operations
 */
export const userError = writable<string | null>(null);

/**
 * List of available usernames
 */
export const availableUsers = writable<string[]>([]);

/**
 * Whether users exist in the system (for first launch detection)
 */
export const hasUsers = writable<boolean>(false);

/**
 * Derived store: Is a user currently logged in?
 */
export const isLoggedIn = derived(currentUser, ($currentUser) => $currentUser !== null);

/**
 * Derived store: Current username (empty string if not logged in)
 */
export const currentUsername = derived(currentUser, ($currentUser) => $currentUser?.username || '');

/**
 * Derived store: User preferences (returns default if not logged in)
 */
export const userPreferences = derived(
  currentUser,
  ($currentUser) =>
    $currentUser?.preferences || {
      theme: 'system' as const,
      default_timeout_ms: 5000,
      inference_mode: 'local' as const,
      language: 'en',
    }
);

/**
 * Check if users exist in the system
 * Called on app start to determine if this is first launch
 *
 * @returns Promise<boolean> - True if users exist, false if empty system
 */
export async function checkHasUsers(): Promise<boolean> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<boolean>>('has_users');

    if (result.success && result.data !== undefined) {
      hasUsers.set(result.data);
      return result.data;
    } else {
      const error = result.error || 'Failed to check for users';
      userError.set(error);
      return false;
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Failed to check for users: ${errorMessage}`);
    return false;
  } finally {
    isLoading.set(false);
  }
}

/**
 * List all available usernames
 * Used in profile selector dropdown
 *
 * @returns Promise<string[]> - Array of usernames
 */
export async function listUsers(): Promise<string[]> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<string[]>>('list_users');

    if (result.success && result.data) {
      availableUsers.set(result.data);
      return result.data;
    } else {
      const error = result.error || 'Failed to list users';
      userError.set(error);
      return [];
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Failed to list users: ${errorMessage}`);
    return [];
  } finally {
    isLoading.set(false);
  }
}

/**
 * Create a new user account
 * Validates username and password, then creates encrypted user directory
 * Automatically logs in the new user after creation
 *
 * @param username - Alphanumeric username (max 32 chars)
 * @param password - Password (min 12 chars)
 * @returns Promise<boolean> - True if user created and logged in successfully
 */
export async function createUser(username: string, password: string): Promise<boolean> {
  try {
    isLoading.set(true);
    userError.set(null);

    // Client-side validation
    const validation = validateUsername(username);
    if (!validation.valid) {
      userError.set(validation.errors.join(', '));
      return false;
    }

    const passwordValidation = validatePassword(password);
    if (!passwordValidation.valid) {
      userError.set(passwordValidation.errors.join(', '));
      return false;
    }

    // Create user via Tauri command
    const result = await invoke<ProfileResult<UserConfig>>('create_user', {
      username,
      password,
    });

    if (result.success && result.data) {
      // User created successfully, store in state
      currentUser.set(result.data);
      hasUsers.set(true);

      // Update available users list
      await listUsers();

      return true;
    } else {
      const error = result.error || 'Failed to create user';
      userError.set(error);
      return false;
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Failed to create user: ${errorMessage}`);
    return false;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Log in an existing user
 * Validates password and loads encrypted user data
 *
 * @param username - Username to log in
 * @param password - User's password
 * @returns Promise<boolean> - True if login successful
 */
export async function loginUser(username: string, password: string): Promise<boolean> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<UserConfig>>('login_user', {
      username,
      password,
    });

    if (result.success && result.data) {
      // Login successful, store user in state
      currentUser.set(result.data);
      return true;
    } else {
      const error = result.error || 'Incorrect password';
      userError.set(error);
      return false;
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Login failed: ${errorMessage}`);
    return false;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Log out the current user
 * Clears user session and sensitive data from memory
 */
export async function logoutUser(): Promise<void> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<void>>('logout_user');

    if (result.success) {
      // Clear user from state
      currentUser.set(null);
    } else {
      const error = result.error || 'Failed to logout';
      userError.set(error);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Logout failed: ${errorMessage}`);
  } finally {
    isLoading.set(false);
  }
}

/**
 * Get current user config
 * Fetches latest user data from backend
 *
 * @returns Promise<UserConfig | null> - Current user config or null if not logged in
 */
export async function getCurrentUser(): Promise<UserConfig | null> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<UserConfig>>('get_current_user');

    if (result.success && result.data) {
      currentUser.set(result.data);
      return result.data;
    } else {
      // Not logged in or error
      return null;
    }
  } catch {
    return null;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Update user profile markdown content
 * Saves the user-profile.md file with new content
 *
 * @param content - Markdown content for user profile
 * @returns Promise<boolean> - True if update successful
 */
export async function updateUserProfile(content: string): Promise<boolean> {
  try {
    isLoading.set(true);
    userError.set(null);

    const result = await invoke<ProfileResult<void>>('update_user_profile', {
      content,
    });

    if (result.success) {
      return true;
    } else {
      const error = result.error || 'Failed to update profile';
      userError.set(error);
      return false;
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    userError.set(`Failed to update profile: ${errorMessage}`);
    return false;
  } finally {
    isLoading.set(false);
  }
}

/**
 * Validate username format
 * Rules:
 * - Alphanumeric, underscore, dash only
 * - 1-32 characters
 * - No spaces or special characters
 *
 * @param username - Username to validate
 * @returns Validation result with errors
 */
export function validateUsername(username: string): { valid: boolean; errors: string[] } {
  const errors: string[] = [];

  if (!username || username.length === 0) {
    errors.push('Username is required');
  }

  if (username.length > 32) {
    errors.push('Username must be 32 characters or less');
  }

  const validPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validPattern.test(username)) {
    errors.push('Username can only contain letters, numbers, underscore, and dash');
  }

  if (/\s/.test(username)) {
    errors.push('Username cannot contain spaces');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validate password strength and requirements
 * Rules:
 * - Minimum 12 characters
 * - Strength indicator: weak/medium/strong
 *
 * @param password - Password to validate
 * @returns Password validation result with strength and suggestions
 */
export function validatePassword(password: string): PasswordValidation {
  const errors: string[] = [];
  const suggestions: string[] = [];

  // Minimum length check
  if (!password || password.length < 12) {
    errors.push('Password must be at least 12 characters');
  }

  // Calculate strength
  let strength: 'weak' | 'medium' | 'strong' = 'weak';
  let score = 0;

  // Length scoring
  if (password.length >= 12) score += 1;
  if (password.length >= 16) score += 1;
  if (password.length >= 20) score += 1;

  // Character variety scoring
  if (/[a-z]/.test(password)) score += 1; // lowercase
  if (/[A-Z]/.test(password)) score += 1; // uppercase
  if (/[0-9]/.test(password)) score += 1; // numbers
  if (/[^a-zA-Z0-9]/.test(password)) score += 1; // special chars

  // Determine strength
  if (score >= 6) {
    strength = 'strong';
  } else if (score >= 4) {
    strength = 'medium';
  } else {
    strength = 'weak';
  }

  // Generate suggestions
  if (password.length < 16) {
    suggestions.push('Use at least 16 characters for better security');
  }
  if (!/[a-z]/.test(password)) {
    suggestions.push('Add lowercase letters');
  }
  if (!/[A-Z]/.test(password)) {
    suggestions.push('Add uppercase letters');
  }
  if (!/[0-9]/.test(password)) {
    suggestions.push('Add numbers');
  }
  if (!/[^a-zA-Z0-9]/.test(password)) {
    suggestions.push('Add special characters (!@#$%^&*)');
  }

  return {
    valid: errors.length === 0,
    strength,
    errors,
    suggestions,
  };
}

/**
 * Clear all user error messages
 */
export function clearUserError(): void {
  userError.set(null);
}

/**
 * Initialize user store on app start
 * Checks if users exist and attempts to restore session
 */
export async function initializeUserStore(): Promise<void> {
  try {
    // Check if users exist
    const usersExist = await checkHasUsers();

    if (usersExist) {
      // List available users for profile selector
      await listUsers();

      // Try to restore session (will fail if no active session)
      await getCurrentUser();
    }
  } catch (error) {
    console.error('Failed to initialize user store:', error);
  }
}
