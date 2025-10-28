<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { Building, DocumentType, DocumentUploadPayload, User } from '../lib/types';
  import { DOCUMENT_TYPE_OPTIONS as DOCUMENT_TYPES } from '../lib/types';
  import { api } from '../lib/api';

  const dispatch = createEventDispatcher();

  export let open = false;
  export let buildings: Building[] = [];
  export let user: User | null = null;
  export let loadingBuildings = false;

  let buildingId = '';
  let documentType: DocumentType = DOCUMENT_TYPES[0].value;
  let title = '';
  let description = '';
  let file: File | null = null;
  let error: string | null = null;
  let submitting = false;
  let fileInput: HTMLInputElement | null = null;

  onMount(() => {
    if (buildings.length > 0) {
      buildingId = buildings[0].id;
    }
  });

  $: if (open && buildings.length > 0 && !buildingId) {
    buildingId = buildings[0].id;
  }

  $: if (buildings.length > 0 && buildingId && !buildings.some((b) => b.id === buildingId)) {
    buildingId = buildings[0].id;
  }

  function resetForm() {
    title = '';
    description = '';
    file = null;
    error = null;
    submitting = false;
    if (fileInput) {
      fileInput.value = '';
    }
  }

  function handleClose() {
    open = false;
    resetForm();
    dispatch('close');
  }

  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    file = target.files && target.files.length > 0 ? target.files[0] : null;
  }

  async function submit(event: Event) {
    event.preventDefault();
    error = null;

    if (!user) {
      error = "Utilisateur non authentifié";
      return;
    }

    if (!file) {
      error = "Veuillez sélectionner un fichier";
      return;
    }

    if (!buildingId) {
      error = "Veuillez sélectionner un bâtiment";
      return;
    }

    if (!title.trim()) {
      error = "Le titre est obligatoire";
      return;
    }

    submitting = true;

    const payload: DocumentUploadPayload = {
      buildingId,
      documentType,
      title: title.trim(),
      description: description.trim() ? description.trim() : undefined,
      file,
      uploadedBy: user.id,
    };

    try {
      const document = await api.uploadDocument(payload);
      dispatch('uploaded', { document });
      handleClose();
    } catch (err) {
      console.error('Upload failed', err);
      error = err instanceof Error ? err.message : "Échec de l'upload";
    } finally {
      submitting = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-40 p-4">
    <div class="bg-white rounded-2xl shadow-xl max-w-xl w-full">
      <form on:submit|preventDefault={submit} class="flex flex-col">
        <div class="px-6 py-4 border-b border-gray-200">
          <div class="flex items-start justify-between">
            <div>
              <h2 class="text-xl font-semibold text-gray-900">Nouveau document</h2>
              <p class="text-sm text-gray-500">Téléversez un document pour la copropriété sélectionnée.</p>
            </div>
            <button
              type="button"
              class="text-gray-400 hover:text-gray-600"
              on:click={handleClose}
              aria-label="Fermer le modal"
            >
              ✕
            </button>
          </div>
        </div>

        <div class="px-6 py-4 space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Bâtiment</label>
            {#if loadingBuildings}
              <p class="text-sm text-gray-500">Chargement des bâtiments…</p>
            {:else if buildings.length === 0}
              <p class="text-sm text-red-500">Aucun bâtiment disponible. Contactez un administrateur.</p>
            {:else}
              <select
                class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500"
                bind:value={buildingId}
              >
                {#each buildings as building}
                  <option value={building.id}>{building.name} · {building.city}</option>
                {/each}
              </select>
            {/if}
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Type de document</label>
            <select
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500"
              bind:value={documentType}
            >
              {#each DOCUMENT_TYPES as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Titre</label>
            <input
              type="text"
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500"
              bind:value={title}
              placeholder="Ex: PV AGO 2024"
              required
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Description (optionnel)</label>
            <textarea
              class="w-full rounded-lg border border-gray-300 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary-500"
              rows={3}
              bind:value={description}
              placeholder="Informations supplémentaires sur le document"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Fichier</label>
            <div class="flex items-center gap-3">
              <input
                type="file"
                class="hidden"
                bind:this={fileInput}
                accept=".pdf,.doc,.docx,.xls,.xlsx,.png,.jpg,.jpeg,.txt"
                on:change={handleFileChange}
              />
              <button
                type="button"
                class="px-3 py-2 rounded-lg border border-gray-300 text-sm font-medium text-gray-700 hover:bg-gray-100 transition"
                on:click={() => fileInput?.click()}
                disabled={submitting}
              >
                Sélectionner un fichier
              </button>
              <span class="text-sm text-gray-600">
                {file ? file.name : 'Aucun fichier sélectionné'}
              </span>
            </div>
            <p class="text-xs text-gray-500 mt-1">Taille maximale 50 Mo.</p>
          </div>

          {#if error}
            <div class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
              {error}
            </div>
          {/if}
        </div>

        <div class="px-6 py-4 bg-gray-50 rounded-b-2xl flex justify-end gap-3">
          <button
            type="button"
            class="px-4 py-2 rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-100 transition"
            on:click={handleClose}
            disabled={submitting}
          >
            Annuler
          </button>
          <button
            type="submit"
            class="px-4 py-2 rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition disabled:opacity-60"
            disabled={submitting || loadingBuildings || buildings.length === 0}
          >
            {submitting ? 'Téléversement…' : 'Téléverser'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
