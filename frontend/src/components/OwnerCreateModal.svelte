<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';
  import type { Organization } from '../lib/types';

  export let isOpen = false;

  const dispatch = createEventDispatcher();

  $: user = $authStore.user;
  $: isSuperAdmin = user?.role === 'superadmin';

  let organizations: Organization[] = [];
  let loadingOrgs = false;

  let formData = {
    organization_id: '',
    first_name: '',
    last_name: '',
    email: '',
    phone: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Belgique',
  };

  $: if (isOpen && isSuperAdmin && organizations.length === 0) {
    loadOrganizations();
  }

  let loading = false;
  let error = '';

  $: if (!isOpen) {
    // Reset form when modal closes
    formData = {
      organization_id: '',
      first_name: '',
      last_name: '',
      email: '',
      phone: '',
      address: '',
      city: '',
      postal_code: '',
      country: 'Belgique',
    };
    error = '';
  }

  async function loadOrganizations() {
    try {
      loadingOrgs = true;
      const response = await api.get<{ data: Organization[] }>('/organizations');
      organizations = response.data;
    } catch (e) {
      console.error('Error loading organizations:', e);
    } finally {
      loadingOrgs = false;
    }
  }

  async function handleSubmit() {
    try {
      loading = true;
      error = '';

      await api.post('/owners', formData);

      dispatch('save');
      closeModal();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de la création';
      console.error('Error creating owner:', e);
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
    <div class="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-xl font-bold text-gray-900">
          Nouveau copropriétaire
        </h2>
      </div>

      <form on:submit|preventDefault={handleSubmit} class="p-6 space-y-4">
        {#if error}
          <div class="bg-red-50 border border-red-200 rounded-lg p-3">
            <p class="text-red-800 text-sm">{error}</p>
          </div>
        {/if}

        {#if isSuperAdmin}
          <div>
            <label for="organization_id" class="block text-sm font-medium text-gray-700 mb-1">
              Organisation *
            </label>
            <select
              id="organization_id"
              bind:value={formData.organization_id}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            >
              <option value="">Sélectionnez une organisation</option>
              {#each organizations as org}
                <option value={org.id}>{org.name}</option>
              {/each}
            </select>
          </div>
        {/if}

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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

        <div>
          <label for="address" class="block text-sm font-medium text-gray-700 mb-1">
            Adresse
          </label>
          <input
            type="text"
            id="address"
            bind:value={formData.address}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label for="postal_code" class="block text-sm font-medium text-gray-700 mb-1">
              Code postal
            </label>
            <input
              type="text"
              id="postal_code"
              bind:value={formData.postal_code}
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <div>
            <label for="city" class="block text-sm font-medium text-gray-700 mb-1">
              Ville
            </label>
            <input
              type="text"
              id="city"
              bind:value={formData.city}
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <div>
            <label for="country" class="block text-sm font-medium text-gray-700 mb-1">
              Pays
            </label>
            <input
              type="text"
              id="country"
              bind:value={formData.country}
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>
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
            {loading ? 'Création...' : 'Créer'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
