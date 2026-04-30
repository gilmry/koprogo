<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import BuildingSelector from './BuildingSelector.svelte';

  let { oncreated, onclose }: {
    oncreated?: () => void;
    onclose?: () => void;
  } = $props();

  let title = $state('');
  let meetingType: 'Ordinary' | 'Extraordinary' = $state('Ordinary');
  let scheduledDate = $state('');
  let location = $state('');
  let description = $state('');
  let buildingId = $state('');
  let loading = $state(false);
  let error = $state('');

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
    if (!location.trim()) {
      error = 'Le lieu est requis';
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
        location: location.trim(),
        description: description.trim() || null,
        building_id: buildingId,
      });
      oncreated?.();
    } catch (err: any) {
      error = err.message || 'Erreur lors de la création';
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    onclose?.();
  }

  function handleBuildingSelect(id: string) {
    buildingId = id;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
  onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
  role="dialog"
  aria-modal="true"
  aria-label="Créer une assemblée"
  tabindex={-1}
>
  <div class="bg-white rounded-lg shadow-xl max-w-lg w-full mx-4 max-h-[90vh] flex flex-col">
    <div class="flex justify-between items-center p-6 pb-4 border-b">
      <h2 class="text-xl font-bold text-gray-900">Nouvelle assemblée générale</h2>
      <button
        onclick={handleClose}
        class="text-gray-400 hover:text-gray-600"
        aria-label="Fermer"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="flex flex-col flex-1 overflow-hidden">
      <div class="overflow-y-auto p-6 space-y-4 flex-1">
    {#if error}
      <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
        {error}
      </div>
    {/if}
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

      <div>
        <label for="meeting-location" class="block text-sm font-medium text-gray-700">
          Lieu <span class="text-red-500">*</span>
        </label>
        <input
          id="meeting-location"
          type="text"
          bind:value={location}
          placeholder="Ex: Salle commune, Résidence du Parc"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          required
          data-testid="input-meeting-location"
        />
      </div>

      <div>
        <label for="meeting-description" class="block text-sm font-medium text-gray-700">
          Description
        </label>
        <textarea
          id="meeting-description"
          bind:value={description}
          rows="3"
          placeholder="Optionnel"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
          data-testid="input-meeting-description"
        ></textarea>
      </div>
      </div>

      <div class="flex justify-end gap-3 p-6 pt-4 border-t bg-gray-50 rounded-b-lg">
        <button
          type="button"
          onclick={handleClose}
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
