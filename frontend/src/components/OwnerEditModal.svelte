<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api';
  import type { Owner } from '../lib/types';

  export let owner: Owner | null = null;
  export let isOpen = false;

  const dispatch = createEventDispatcher();

  let formData = {
    first_name: '',
    last_name: '',
    email: '',
    phone: '',
  };

  let loading = false;
  let error = '';

  $: if (owner && isOpen) {
    formData = {
      first_name: owner.first_name,
      last_name: owner.last_name,
      email: owner.email,
      phone: owner.phone || '',
    };
  }

  async function handleSubmit() {
    if (!owner) return;

    try {
      loading = true;
      error = '';

      await api.put(`/owners/${owner.id}`, formData);

      dispatch('save');
      closeModal();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de la modification';
      console.error('Error updating owner:', e);
    } finally {
      loading = false;
    }
  }

  function closeModal() {
    dispatch('close');
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      closeModal();
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    on:click={handleBackdropClick}
  >
    <div class="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-xl font-bold text-gray-900">
          Modifier le copropriétaire
        </h2>
      </div>

      <form on:submit|preventDefault={handleSubmit} class="p-6 space-y-4">
        {#if error}
          <div class="bg-red-50 border border-red-200 rounded-lg p-3">
            <p class="text-red-800 text-sm">{error}</p>
          </div>
        {/if}

        <div>
          <label for="first_name" class="block text-sm font-medium text-gray-700 mb-1">
            Prénom *
          </label>
          <input
            type="text"
            id="first_name"
            bind:value={formData.first_name}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label for="last_name" class="block text-sm font-medium text-gray-700 mb-1">
            Nom *
          </label>
          <input
            type="text"
            id="last_name"
            bind:value={formData.last_name}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label for="email" class="block text-sm font-medium text-gray-700 mb-1">
            Email *
          </label>
          <input
            type="email"
            id="email"
            bind:value={formData.email}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label for="phone" class="block text-sm font-medium text-gray-700 mb-1">
            Téléphone
          </label>
          <input
            type="tel"
            id="phone"
            bind:value={formData.phone}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div class="flex justify-end space-x-3 pt-4">
          <button
            type="button"
            on:click={closeModal}
            disabled={loading}
            class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition disabled:opacity-50"
          >
            Annuler
          </button>
          <button
            type="submit"
            disabled={loading}
            class="px-4 py-2 text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition disabled:opacity-50"
          >
            {loading ? 'Enregistrement...' : 'Enregistrer'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
