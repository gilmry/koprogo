<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';

  let isOpen = false;

  onMount(() => {
    // Check if consent has been given
    const consentAccepted = localStorage.getItem('consent-accepted');
    if (!consentAccepted) {
      isOpen = true;
    }
  });

  function handleAccept() {
    // Set consent flag in localStorage
    localStorage.setItem('consent-accepted', 'true');
    isOpen = false;

    // Optionally: record consent on backend (async, no await needed)
    recordConsent();
  }

  async function recordConsent() {
    try {
      const token = localStorage.getItem('auth_token');
      if (!token) return; // Only record if user is authenticated

      const response = await fetch('/api/v1/consent', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify({
          consent_type: 'privacy_policy',
        }),
      });

      if (!response.ok) {
        console.warn('Failed to record consent on backend');
      }
    } catch (err) {
      console.warn('Error recording consent:', err);
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black bg-opacity-50 z-40" on:click={() => {}} />
  <div class="fixed bottom-0 left-0 right-0 bg-white rounded-t-lg shadow-2xl z-50 p-6 sm:rounded-lg sm:bottom-auto sm:left-1/2 sm:-translate-x-1/2 sm:top-1/2 sm:-translate-y-1/2 sm:max-w-lg sm:mx-auto">
    <div class="space-y-4">
      <h2 class="text-xl font-bold text-gray-900">
        {$_('privacy.consent.title')}
      </h2>

      <p class="text-sm text-gray-600">
        {$_('privacy.consent.message')}
        <a
          href="/privacy-policy"
          target="_blank"
          rel="noopener noreferrer"
          class="text-blue-600 hover:underline font-medium"
          data-testid="consent-modal-privacy-link"
        >
          {$_('privacy.consent.linkText')}
        </a>
      </p>

      <div class="pt-4 flex gap-3 justify-end">
        <button
          on:click={handleAccept}
          class="px-6 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 transition-colors"
          data-testid="consent-modal-accept-btn"
        >
          {$_('privacy.consent.accept')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body.no-scroll) {
    overflow: hidden;
  }
</style>
