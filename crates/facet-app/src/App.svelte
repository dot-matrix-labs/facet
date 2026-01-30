<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { setupEventListeners, cleanupEventListeners } from './lib/events';
  import { initializeConsoleLogger } from './lib/logger';
  import ChatInterface from './components/ChatInterface.svelte';
  import DebugView from './components/DebugView.svelte';
  import DeveloperMode from './components/DeveloperMode.svelte';
  import UpdateModal from './components/UpdateModal.svelte';
  import LoginScreen from './components/LoginScreen.svelte';
  import UserCreationForm from './components/UserCreationForm.svelte';
  import ProfileSwitcher from './components/ProfileSwitcher.svelte';
  import ProfileEditor from './components/ProfileEditor.svelte';
  import CommandManager from './components/CommandManager.svelte';
  import FeedbackDialog from './components/FeedbackDialog.svelte';
  import { initializeUserStore, isLoggedIn, hasUsers, listUsers } from './lib/userStore';

  type ViewType = 'chat' | 'debug' | 'developer' | 'profile-editor' | 'commands';
  type AuthViewType = 'login' | 'create-user';

  let currentView: ViewType = 'chat';
  let authView: AuthViewType = 'login';
  let updateModalRef: UpdateModal;
  let menuOpen = false;
  let showFeedbackDialog = false;

  // Subscribe to auth state
  let loggedIn = false;
  let usersExist = false;

  const unsubLoggedIn = isLoggedIn.subscribe((value) => {
    loggedIn = value;
  });

  const unsubHasUsers = hasUsers.subscribe((value) => {
    usersExist = value;
  });

  onMount(async () => {
    // Initialize console logger (intercepts console.log, etc.)
    initializeConsoleLogger();

    // Initialize user store (check for users, restore session)
    await initializeUserStore();

    // Setup event listeners for debug events
    await setupEventListeners();

    // Position window (right 1/3 of screen)
    await positionWindow();
  });

  onDestroy(() => {
    // Cleanup event listeners
    cleanupEventListeners();

    // Cleanup auth subscriptions
    unsubLoggedIn();
    unsubHasUsers();
  });

  async function positionWindow() {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const { currentMonitor } = await import('@tauri-apps/api/window');
      const { LogicalSize, LogicalPosition } = await import('@tauri-apps/api/dpi');

      // Wait a bit for window to be ready
      await new Promise((resolve) => setTimeout(resolve, 200));

      const appWindow = getCurrentWebviewWindow();
      const monitor = await currentMonitor();
      if (!monitor) {
        console.warn('Could not get current monitor');
        return;
      }

      // Get monitor position and size
      const monitorX = monitor.position.x;
      const monitorY = monitor.position.y;
      const screenWidth = monitor.size.width;
      const screenHeight = monitor.size.height;

      console.log(`Monitor: ${screenWidth}x${screenHeight} at (${monitorX}, ${monitorY})`);

      // App window: right 1/4 of screen, full vertical height
      // Use minimum of 400px to ensure it's usable on smaller screens
      const appWidth = Math.max(400, Math.floor(screenWidth / 4));

      // Calculate position relative to monitor position
      const appX = monitorX + screenWidth - appWidth;
      const appY = monitorY;

      console.log(`Positioning window: ${appWidth}x${screenHeight} at (${appX}, ${appY})`);

      // Set size first, then position
      await appWindow.setSize(new LogicalSize(appWidth, screenHeight));
      await new Promise((resolve) => setTimeout(resolve, 50));
      await appWindow.setPosition(new LogicalPosition(appX, appY));

      console.log('Window positioned successfully');
    } catch (error) {
      console.error('Failed to position window:', error);
    }
  }

  function handleCheckForUpdates() {
    if (updateModalRef) {
      updateModalRef.checkNow();
    }
    menuOpen = false;
  }

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  function handleViewChange(view: ViewType) {
    currentView = view;
    menuOpen = false;
  }

  // Close menu when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (menuOpen && !target.closest('.menu-container')) {
      menuOpen = false;
    }
  }

  /**
   * Handle successful login
   * Navigate to main app view
   */
  function handleLoginSuccess() {
    currentView = 'chat';
  }

  /**
   * Handle user creation flow
   * Show user creation form
   */
  function handleCreateUser() {
    authView = 'create-user';
  }

  /**
   * Handle successful user creation
   * User is automatically logged in after creation
   */
  function handleUserCreated() {
    currentView = 'chat';
  }

  /**
   * Handle back from user creation to login
   */
  function handleBackToLogin() {
    authView = 'login';
  }

  /**
   * Handle logout
   * Show login screen
   */
  function handleLogout() {
    authView = 'login';
    currentView = 'chat';
    menuOpen = false;
  }

  /**
   * Handle switch profile
   * Show login screen with user list
   */
  function handleSwitchProfile() {
    authView = 'login';
    currentView = 'chat';
    menuOpen = false;
    // Refresh user list
    listUsers();
  }

  /**
   * Handle edit profile
   * Navigate to profile editor
   */
  function handleEditProfile() {
    currentView = 'profile-editor';
    menuOpen = false;
  }
</script>

<svelte:window on:click={handleClickOutside} />

<!-- Authentication Views (shown when not logged in) -->
{#if !loggedIn}
  {#if !usersExist || authView === 'create-user'}
    <!-- No users exist (first launch) or user clicked "Create New User" -->
    <UserCreationForm
      showBackButton={usersExist}
      on:userCreated={handleUserCreated}
      on:back={handleBackToLogin}
    />
  {:else}
    <!-- Users exist, show login screen -->
    <LoginScreen on:loginSuccess={handleLoginSuccess} on:createUser={handleCreateUser} />
  {/if}
{:else}
  <!-- Main Application (shown when logged in) -->
  <div class="app-container">
    <div class="menu-container">
      <button class="cog-button" on:click={toggleMenu} title="Menu">
        <svg
          width="20"
          height="20"
          viewBox="0 0 20 20"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M10 6C8.9 6 8 6.9 8 8C8 9.1 8.9 10 10 10C11.1 10 12 9.1 12 8C12 6.9 11.1 6 10 6ZM10 2C8.9 2 8 2.9 8 4C8 5.1 8.9 6 10 6C11.1 6 12 5.1 12 4C12 2.9 11.1 2 10 2ZM10 10C8.9 10 8 10.9 8 12C8 13.1 8.9 14 10 14C11.1 14 12 13.1 12 12C12 10.9 11.1 10 10 10Z"
            fill="currentColor"
          />
        </svg>
      </button>

      {#if menuOpen}
        <div class="dropdown-menu">
          <div class="menu-header">
            <span class="menu-title">Robert</span>
          </div>

          <!-- Profile Switcher (user info and auth actions) -->
          <ProfileSwitcher
            on:logout={handleLogout}
            on:switchProfile={handleSwitchProfile}
            on:editProfile={handleEditProfile}
          />

          <div class="menu-divider"></div>

          <button
            class="menu-item"
            class:active={currentView === 'chat'}
            on:click={() => handleViewChange('chat')}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M14 2H2C1.45 2 1 2.45 1 3V11C1 11.55 1.45 12 2 12H4V15L8 12H14C14.55 12 15 11.55 15 11V3C15 2.45 14.55 2 14 2Z"
                stroke="currentColor"
                stroke-width="1.5"
                fill="none"
              />
            </svg>
            <span>Chat</span>
          </button>
          <button
            class="menu-item"
            class:active={currentView === 'debug'}
            on:click={() => handleViewChange('debug')}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path d="M3 2H13V14H3V2Z" stroke="currentColor" stroke-width="1.5" fill="none" />
              <line x1="5" y1="5" x2="11" y2="5" stroke="currentColor" stroke-width="1.5" />
              <line x1="5" y1="8" x2="11" y2="8" stroke="currentColor" stroke-width="1.5" />
              <line x1="5" y1="11" x2="9" y2="11" stroke="currentColor" stroke-width="1.5" />
            </svg>
            <span>Debug Log</span>
          </button>
          <button
            class="menu-item"
            class:active={currentView === 'developer'}
            on:click={() => handleViewChange('developer')}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M5 4L2 8L5 12M11 4L14 8L11 12M9 2L7 14"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            <span>Developer</span>
          </button>
          <button
            class="menu-item"
            class:active={currentView === 'commands'}
            on:click={() => handleViewChange('commands')}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M2 3H14V5H2V3ZM2 7H14V9H2V7ZM2 11H14V13H2V11Z"
                stroke="currentColor"
                stroke-width="1.5"
                fill="none"
              />
            </svg>
            <span>Commands</span>
          </button>
          <div class="menu-divider"></div>

          <button
            class="menu-item"
            on:click={() => {
              showFeedbackDialog = true;
              menuOpen = false;
            }}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M13.5 2.5V10.5C13.5 11.05 13.05 11.5 12.5 11.5H4L1 14.5V2.5C1 1.95 1.45 1.5 2 1.5H12.5C13.05 1.5 13.5 1.95 13.5 2.5Z"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M7.25 8.5H7.26"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
              <path
                d="M7.25 4.5V6.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            <span>Send Feedback</span>
          </button>

          <button class="menu-item" on:click={handleCheckForUpdates}>
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M14 8C14 11.3137 11.3137 14 8 14C4.68629 14 2 11.3137 2 8C2 4.68629 4.68629 2 8 2C9.5 2 10.84 2.58 11.84 3.5"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
              <path
                d="M11 2V4H13"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
            <span>Check for Updates</span>
          </button>
        </div>
      {/if}
    </div>

    <main>
      {#if currentView === 'chat'}
        <ChatInterface />
      {:else if currentView === 'debug'}
        <DebugView />
      {:else if currentView === 'developer'}
        <DeveloperMode />
      {:else if currentView === 'profile-editor'}
        <ProfileEditor />
      {:else if currentView === 'commands'}
        <CommandManager />
      {/if}
    </main>

    <!-- Update Modal with auto-check enabled -->
    <UpdateModal bind:this={updateModalRef} autoCheck={true} />

    <FeedbackDialog bind:show={showFeedbackDialog} on:close={() => (showFeedbackDialog = false)} />
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background: #ffffff;
    overflow: hidden;
  }

  .app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .menu-container {
    position: absolute;
    top: 1.875rem;
    right: 1.5rem;
    z-index: 1000;
  }

  .cog-button {
    background: white;
    border: 1px solid #d1d5db;
    color: #4b5563;
    padding: 0.5rem;
    border-radius: 6px;
    cursor: pointer;
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .cog-button:hover {
    background: #f9fafb;
    border-color: #9ca3af;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.12);
  }

  .cog-button:active {
    background: #f3f4f6;
  }

  .dropdown-menu {
    position: absolute;
    top: 48px;
    right: 0;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    box-shadow:
      0 10px 25px rgba(0, 0, 0, 0.1),
      0 4px 10px rgba(0, 0, 0, 0.05);
    min-width: 220px;
    overflow: hidden;
    animation: slideDown 0.15s ease;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .menu-header {
    padding: 0.875rem 1rem;
    background: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
  }

  .menu-title {
    font-weight: 600;
    font-size: 0.875rem;
    color: #111827;
    letter-spacing: 0.025em;
    text-transform: uppercase;
  }

  .menu-divider {
    height: 1px;
    background: #e5e7eb;
    margin: 0.25rem 0;
  }

  .menu-item {
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

  .menu-item svg {
    flex-shrink: 0;
    opacity: 0.7;
  }

  .menu-item:hover {
    background: #f3f4f6;
    color: #111827;
  }

  .menu-item:hover svg {
    opacity: 1;
  }

  .menu-item.active {
    background: #eef2ff;
    color: #4f46e5;
  }

  .menu-item.active svg {
    opacity: 1;
  }

  main {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
