<script lang="ts">
  import { onMount } from 'svelte';
  import { syncService } from '../lib/sync';

  let isOnline = true;
  let syncing = false;

  onMount(() => {
    isOnline = syncService.getOnlineStatus();

    // Listen for online/offline events
    const handleOnline = () => {
      isOnline = true;
    };

    const handleOffline = () => {
      isOnline = false;
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  });

  async function handleSync() {
    if (!isOnline || syncing) return;

    syncing = true;
    try {
      await syncService.sync();
    } catch (error) {
      console.error('Sync failed:', error);
    } finally {
      syncing = false;
    }
  }
</script>

<div class="flex items-center gap-3">
  <!-- Online/Offline indicator -->
  <div class="flex items-center gap-2">
    {#if isOnline}
      <div class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
      <span class="text-sm text-gray-600">En ligne</span>
    {:else}
      <div class="w-3 h-3 bg-red-500 rounded-full"></div>
      <span class="text-sm text-gray-600">Hors ligne</span>
    {/if}
  </div>

  <!-- Sync button -->
  {#if isOnline}
    <button
      on:click={handleSync}
      disabled={syncing}
      class="flex items-center gap-2 px-3 py-1.5 text-sm bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition disabled:opacity-50 disabled:cursor-not-allowed"
      title="Synchroniser les données"
    >
      <svg
        class="w-4 h-4 {syncing ? 'animate-spin' : ''}"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
        />
      </svg>
      {syncing ? 'Sync...' : 'Sync'}
    </button>
  {/if}
</div>
