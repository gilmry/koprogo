<script lang="ts">
  import { onMount } from 'svelte';
  import { showInstallPrompt, canInstall, isPWA } from '../lib/pwa';

  let showPrompt = false;
  let isInstalled = false;
  let showUpdateNotification = false;

  onMount(() => {
    // Check if already installed
    isInstalled = isPWA();

    // Listen for install prompt availability
    window.addEventListener('pwa-installable', () => {
      showPrompt = true;
    });

    // Listen for installation
    window.addEventListener('pwa-installed', () => {
      isInstalled = true;
      showPrompt = false;
    });

    // Listen for updates
    window.addEventListener('pwa-update-available', () => {
      showUpdateNotification = true;
    });

    // Check if can install now
    if (canInstall() && !isInstalled) {
      showPrompt = true;
    }
  });

  async function handleInstall() {
    const accepted = await showInstallPrompt();
    if (accepted) {
      showPrompt = false;
    }
  }

  function dismissPrompt() {
    showPrompt = false;
    // Remember dismissal for 7 days
    localStorage.setItem('pwa-install-dismissed', Date.now().toString());
  }

  function reloadForUpdate() {
    window.location.reload();
  }

  function dismissUpdate() {
    showUpdateNotification = false;
  }
</script>

<!-- Install Prompt -->
{#if showPrompt && !isInstalled}
  <div class="fixed bottom-4 left-4 right-4 md:left-auto md:right-4 md:w-96 z-50 animate-slide-up">
    <div class="bg-white rounded-lg shadow-2xl border border-gray-200 p-4">
      <div class="flex items-start gap-3">
        <div class="flex-shrink-0">
          <svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="text-lg font-semibold text-gray-900 mb-1">
            Installer KoproGo
          </h3>
          <p class="text-sm text-gray-600 mb-4">
            Installez l'application sur votre appareil pour un accès rapide et un mode hors ligne.
          </p>
          <div class="flex gap-2">
            <button
              on:click={handleInstall}
              class="flex-1 bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
            >
              Installer
            </button>
            <button
              on:click={dismissPrompt}
              class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
            >
              Plus tard
            </button>
          </div>
        </div>
        <button
          on:click={dismissPrompt}
          class="flex-shrink-0 text-gray-400 hover:text-gray-600 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Update Notification -->
{#if showUpdateNotification}
  <div class="fixed top-4 left-4 right-4 md:left-auto md:right-4 md:w-96 z-50 animate-slide-down">
    <div class="bg-blue-600 text-white rounded-lg shadow-2xl p-4">
      <div class="flex items-start gap-3">
        <div class="flex-shrink-0">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="font-semibold mb-1">
            Mise à jour disponible
          </h3>
          <p class="text-sm text-blue-100 mb-3">
            Une nouvelle version de KoproGo est disponible.
          </p>
          <div class="flex gap-2">
            <button
              on:click={reloadForUpdate}
              class="flex-1 bg-white text-blue-600 hover:bg-blue-50 px-4 py-2 rounded-lg font-medium transition-colors"
            >
              Actualiser
            </button>
            <button
              on:click={dismissUpdate}
              class="px-4 py-2 text-blue-100 hover:text-white transition-colors"
            >
              Plus tard
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  @keyframes slide-up {
    from {
      transform: translateY(100%);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  @keyframes slide-down {
    from {
      transform: translateY(-100%);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .animate-slide-up {
    animation: slide-up 0.3s ease-out;
  }

  .animate-slide-down {
    animation: slide-down 0.3s ease-out;
  }
</style>
