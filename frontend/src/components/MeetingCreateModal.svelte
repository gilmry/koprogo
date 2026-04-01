<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import BuildingSelector from './BuildingSelector.svelte';

  const dispatch = createEventDispatcher();

  let title = '';
  let meetingType: 'Ordinary' | 'Extraordinary' = 'Ordinary';
  let scheduledDate = '';
  let buildingId = '';
  let loading = false;
  let error = '';

  async function handleSubmit() {
    error = '';

    if (!title.trim()) {
      error = 'Le titre est requis';
      return;
    }
    if (!scheduledDate) {
      error = 'La date est requise';
      return;
    }
    if (!buildingId) {
      error = "Veuillez sélectionner un immeuble";
      return;
    }

    loading = true;
    try {
      await api.post('/meetings', {
        title: title.trim(),
        meeting_type: meetingType,
        scheduled_date: new Date(scheduledDate).toISOString(),
        building_id: buildingId,
      });
      dispatch('created');
    } catch (err: any) {
      error = err.message || 'Erreur lors de la création';
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    dispatch('close');
  }

  function handleBuildingSelect(id: string) {
    buildingId = id;
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  on:click|self={handleClose}
  role="dialog"
  aria-modal="true"
  aria-label="Créer une assemblée"
>
  <div class="bg-white rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-xl font-bold text-gray-900">Nouvelle assemblée générale</h2>
      <button
        on:click={handleClose}
        class="text-gray-400 hover:text-gray-600"
        aria-label="Fermer"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>

    {#if error}
      <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
        {error}
      </div>
    {/if}

    <form on:submit|preventDefault={handleSubmit} class="space-y-4">
      <BuildingSelector
        bind:selectedBuildingId={buildingId}
        onSelect={handleBuildingSelect}
        label="Immeuble"
        required={true}
      />

      <div>
        <label for="meeting-title" class="block text-sm font-medium text-gray-700">
          Titre <span class="text-red-500">*</span>
        </label>
        <input
          id="meeting-title"
          type="text"
          bind:value={title}
          placeholder="Ex: AG Ordinaire 2026"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          required
          data-testid="input-meeting-title"
        />
      </div>

      <div>
        <label for="meeting-type" class="block text-sm font-medium text-gray-700">
          Type d'assemblée <span class="text-red-500">*</span>
        </label>
        <select
          id="meeting-type"
          bind:value={meetingType}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          data-testid="select-meeting-type"
        >
          <option value="Ordinary">Assemblée Ordinaire</option>
          <option value="Extraordinary">Assemblée Extraordinaire</option>
        </select>
      </div>

      <div>
        <label for="meeting-date" class="block text-sm font-medium text-gray-700">
          Date et heure <span class="text-red-500">*</span>
        </label>
        <input
          id="meeting-date"
          type="datetime-local"
          bind:value={scheduledDate}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          required
          data-testid="input-meeting-date"
        />
      </div>

      <div class="flex justify-end gap-3 pt-4 border-t">
        <button
          type="button"
          on:click={handleClose}
          class="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
        >
          Annuler
        </button>
        <button
          type="submit"
          disabled={loading}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition disabled:opacity-50"
          data-testid="btn-submit-meeting"
        >
          {#if loading}
            Création...
          {:else}
            Créer l'assemblée
          {/if}
        </button>
      </div>
    </form>
  </div>
</div>
