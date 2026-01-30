<script lang="ts">
  /**
   * Profile Switcher Component
   *
   * Displays current user info and provides logout/switch profile options.
   * Integrated into the app menu dropdown.
   *
   * Features:
   * - Shows current username
   * - Logout button
   * - Switch profile button (shows confirmation if needed)
   * - Displays user stats (commands run, sessions)
   * - Opens profile editor
   */

  import { currentUser, logoutUser } from '../lib/userStore';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  // Subscribe to current user
  let user: typeof $currentUser = null;
  const unsubUser = currentUser.subscribe((value) => {
    user = value;
  });

  /**
   * Handle logout
   * Dispatches event for parent to handle navigation
   */
  async function handleLogout() {
    await logoutUser();
    dispatch('logout');
  }

  /**
   * Handle switch profile
   * Logs out current user and shows login screen
   */
  async function handleSwitchProfile() {
    await logoutUser();
    dispatch('switchProfile');
  }

  /**
   * Open profile editor
   */
  function handleEditProfile() {
    dispatch('editProfile');
  }

  // Cleanup subscriptions
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    unsubUser();
  });
</script>

{#if user}
  <div class="profile-switcher">
    <!-- User Info -->
    <div class="user-info">
      <div class="user-avatar">
        <svg
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="2" />
          <path
            d="M4 20C4 16.6863 6.68629 14 10 14H14C17.3137 14 20 16.6863 20 20"
            stroke="currentColor"
            stroke-width="2"
          />
        </svg>
      </div>
      <div class="user-details">
        <div class="user-name">{user.username}</div>
        <div class="user-stats">
          {user.stats.total_commands_run} commands • {user.stats.total_sessions} sessions
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="profile-actions">
      <button class="profile-action-button" on:click={handleEditProfile}>
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M11.5 2.5L13.5 4.5L11.5 2.5ZM12.75 1.25L9 5L8 8L11 7L14.75 3.25C15.08 2.92 15.08 2.42 14.75 2.08L13.92 1.25C13.58 0.92 13.08 0.92 12.75 1.25ZM1 15H15"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>Edit Profile</span>
      </button>

      <button class="profile-action-button" on:click={handleSwitchProfile}>
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M10 3H14V7M14 3L9 8M6 13H2V9M2 13L7 8"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>Switch Profile</span>
      </button>

      <button class="profile-action-button logout" on:click={handleLogout}>
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M6 14H3C2.45 14 2 13.55 2 13V3C2 2.45 2.45 2 3 2H6M11 11L14 8M14 8L11 5M14 8H6"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>Sign Out</span>
      </button>
    </div>
  </div>
{/if}

<style>
  .profile-switcher {
    padding: 0.5rem 0;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem 1rem;
    background: #f9fafb;
    border-radius: 8px;
    margin: 0.5rem;
  }

  .user-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    flex-shrink: 0;
  }

  .user-details {
    flex: 1;
    min-width: 0;
  }

  .user-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: #111827;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-stats {
    font-size: 0.75rem;
    color: #6b7280;
    margin-top: 0.125rem;
  }

  .profile-actions {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    padding: 0.25rem 0;
  }

  .profile-action-button {
    width: 100%;
    padding: 0.625rem 1rem;
    background: white;
    border: none;
    color: #374151;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.1s ease;
    display: flex;
    align-items: center;
    gap: 0.625rem;
    text-align: left;
  }

  .profile-action-button svg {
    flex-shrink: 0;
    opacity: 0.7;
  }

  .profile-action-button:hover {
    background: #f3f4f6;
    color: #111827;
  }

  .profile-action-button:hover svg {
    opacity: 1;
  }

  .profile-action-button.logout {
    color: #dc2626;
  }

  .profile-action-button.logout:hover {
    background: #fef2f2;
    color: #991b1b;
  }
</style>
