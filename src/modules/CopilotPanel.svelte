<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { copilotGetStatus, copilotSignIn, copilotSignOut, type CopilotStatus } from '$lib/copilotCommands';

  let status: CopilotStatus = {
    status: 'Inactive',
    message: 'Not started',
    signedIn: false,
  };
  
  let showAuthDialog = false;
  let userCode = '';
  let verificationUri = '';
  let isLoading = false;

  let statusInterval: number;

  onMount(async () => {
    await updateStatus();
    
    // Poll status every 5 seconds
    statusInterval = window.setInterval(updateStatus, 5000);
  });

  onDestroy(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
  });

  async function updateStatus() {
    try {
      status = await copilotGetStatus();
    } catch (error) {
      console.error('Failed to get Copilot status:', error);
    }
  }

  async function handleSignIn() {
    isLoading = true;
    try {
      const response = await copilotSignIn();
      userCode = response.userCode;
      verificationUri = response.verificationUri;
      showAuthDialog = true;

      // Open browser
      const { open } = await import('@tauri-apps/api/shell');
      await open(verificationUri);
    } catch (error) {
      console.error('Failed to sign in:', error);
      alert(`Failed to sign in: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  async function handleSignOut() {
    isLoading = true;
    try {
      await copilotSignOut();
      await updateStatus();
    } catch (error) {
      console.error('Failed to sign out:', error);
      alert(`Failed to sign out: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  function closeAuthDialog() {
    showAuthDialog = false;
    userCode = '';
    verificationUri = '';
  }

  function getStatusColor(): string {
    switch (status.status) {
      case 'Normal': return 'text-green-400';
      case 'Warning': return 'text-yellow-400';
      case 'Error': return 'text-red-400';
      case 'Inactive': return 'text-gray-400';
      default: return 'text-gray-400';
    }
  }

  function getStatusIcon(): string {
    switch (status.status) {
      case 'Normal': return '✓';
      case 'Warning': return '⚠';
      case 'Error': return '✗';
      case 'Inactive': return '○';
      default: return '○';
    }
  }
</script>

<div class="copilot-status-panel bg-black border border-gray-800 rounded-lg p-4 w-80">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-white font-semibold text-lg">GitHub Copilot</h3>
    <span class={`${getStatusColor()} font-mono text-sm`}>
      {getStatusIcon()} {status.status}
    </span>
  </div>

  <div class="mb-4">
    <p class="text-gray-400 text-sm">{status.message}</p>
  </div>

  <div class="flex gap-2">
    {#if !status.signedIn}
      <button
        on:click={handleSignIn}
        disabled={isLoading}
        class="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 text-white px-4 py-2 rounded transition-colors text-sm font-medium"
      >
        {isLoading ? 'Loading...' : 'Sign In'}
      </button>
    {:else}
      <button
        on:click={handleSignOut}
        disabled={isLoading}
        class="flex-1 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 text-white px-4 py-2 rounded transition-colors text-sm font-medium"
      >
        {isLoading ? 'Loading...' : 'Sign Out'}
      </button>
    {/if}
  </div>

  <div class="mt-4 text-xs text-gray-500">
    <p>Status updates every 5 seconds</p>
  </div>
</div>

{#if showAuthDialog}
  <div class="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
    <div class="bg-[#0b0f14] border border-gray-800 rounded-lg p-6 w-96 max-w-full">
      <h3 class="text-white font-semibold text-xl mb-4">Sign in to GitHub Copilot</h3>
      
      <div class="mb-4">
        <p class="text-gray-400 text-sm mb-2">
          Copy this code and paste it in the browser window that just opened:
        </p>
        <div class="bg-black border border-gray-700 rounded p-3 text-center">
          <code class="text-white text-2xl font-mono tracking-wider">{userCode}</code>
        </div>
      </div>

      <div class="mb-4">
        <p class="text-gray-400 text-xs">
          If the browser didn't open, visit: 
          <a href={verificationUri} class="text-blue-400 hover:text-blue-300 underline" target="_blank" rel="noopener noreferrer">
            {verificationUri}
          </a>
        </p>
      </div>

      <div class="flex gap-2">
        <button
          on:click={() => navigator.clipboard.writeText(userCode)}
          class="flex-1 bg-gray-700 hover:bg-gray-600 text-white px-4 py-2 rounded transition-colors text-sm font-medium"
        >
          Copy Code
        </button>
        <button
          on:click={closeAuthDialog}
          class="flex-1 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded transition-colors text-sm font-medium"
        >
          Done
        </button>
      </div>

      <p class="text-gray-500 text-xs mt-4 text-center">
        After signing in, wait a few seconds for status to update
      </p>
    </div>
  </div>
{/if}

<style>
  /* Tailwind CSS classes used above */
</style>
