<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { syncService } from '../lib/sync';

  let isOnline = true;
  let syncing = false;

  // Ces contrôles ne servent qu'à l'application installée : dans un onglet
  // ordinaire, le navigateur ne met rien hors ligne et « Sync » n'a rien à
  // synchroniser. Les afficher pour tout le monde exposait de la plomberie
  // PWA dans le pied de page de chaque page publique.
  let isStandalone = false;

  onMount(() => {
    isStandalone =
      window.matchMedia('(display-mode: standalone)').matches ||
      // iOS Safari, qui n'implémente pas display-mode.
      (window.navigator as unknown as { standalone?: boolean }).standalone === true;
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
  <!-- Hors ligne est signalé à tout le monde : l'information est utile même
       dans un onglet ordinaire, où elle explique pourquoi rien ne charge.
       L'état « en ligne », lui, ne dit rien que l'utilisateur ne sache déjà. -->
  {#if !isOnline || isStandalone}
    <div class="flex items-center gap-2">
      {#if isOnline}
        <div class="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
        <span class="text-sm text-gray-600">{$_('sync.online')}</span>
      {:else}
        <div class="w-3 h-3 bg-red-500 rounded-full"></div>
        <span class="text-sm text-gray-600">{$_('sync.offline')}</span>
      {/if}
    </div>
  {/if}

  <!-- Sync button -->
  {#if isOnline && isStandalone}
    <button
      on:click={handleSync}
      disabled={syncing}
      class="flex items-center gap-2 px-3 py-1.5 text-sm bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition disabled:opacity-50 disabled:cursor-not-allowed"
      aria-label={$_('sync.ariaLabel')}
      title={$_('sync.ariaLabel')}
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
      {syncing ? $_('sync.inProgress') : $_('sync.button')}
    </button>
  {/if}
</div>
