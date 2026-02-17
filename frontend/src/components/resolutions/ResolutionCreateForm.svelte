<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    resolutionsApi,
    type Resolution,
    MajorityType,
  } from '../../lib/api/resolutions';
  import { toast } from '../../stores/toast';

  export let meetingId: string;

  const dispatch = createEventDispatcher<{ created: Resolution }>();

  let title = '';
  let description = '';
  let resolutionType = 'standard';
  let majorityRequired: MajorityType = MajorityType.Simple;
  let loading = false;

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error('Le titre est obligatoire');
      return;
    }

    try {
      loading = true;
      const resolution = await resolutionsApi.create(meetingId, {
        meeting_id: meetingId,
        title: title.trim(),
        description: description.trim(),
        resolution_type: resolutionType,
        majority_required: majorityRequired,
      });
      toast.success('Résolution créée avec succès');
      dispatch('created', resolution);
      // Reset form
      title = '';
      description = '';
      resolutionType = 'standard';
      majorityRequired = MajorityType.Simple;
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la création');
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="bg-gray-50 border border-gray-200 rounded-lg p-4">
  <h4 class="text-sm font-semibold text-gray-900 mb-3">Nouvelle résolution</h4>

  <div class="space-y-3">
    <div>
      <label for="resolution-title" class="block text-xs font-medium text-gray-700 mb-1">
        Titre *
      </label>
      <input
        id="resolution-title"
        type="text"
        bind:value={title}
        placeholder="ex: Approbation des comptes 2025"
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
      />
    </div>

    <div>
      <label for="resolution-description" class="block text-xs font-medium text-gray-700 mb-1">
        Description
      </label>
      <textarea
        id="resolution-description"
        bind:value={description}
        rows="2"
        placeholder="Détails de la résolution..."
        class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
      ></textarea>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label for="resolution-type" class="block text-xs font-medium text-gray-700 mb-1">
          Type
        </label>
        <select
          id="resolution-type"
          bind:value={resolutionType}
          class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
        >
          <option value="standard">Standard</option>
          <option value="budget">Budget / Comptes</option>
          <option value="works">Travaux</option>
          <option value="rules">Règlement</option>
          <option value="election">Élection</option>
          <option value="other">Autre</option>
        </select>
      </div>

      <div>
        <label for="resolution-majority" class="block text-xs font-medium text-gray-700 mb-1">
          Majorité requise
        </label>
        <select
          id="resolution-majority"
          bind:value={majorityRequired}
          class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
        >
          <option value={MajorityType.Simple}>Simple (50%+1 exprimés)</option>
          <option value={MajorityType.Absolute}>Absolue (50%+1 de tous)</option>
          <option value={MajorityType.Qualified}>Qualifiée (2/3, 3/4...)</option>
        </select>
      </div>
    </div>

    <div class="p-3 bg-yellow-50 border border-yellow-200 rounded-md text-xs text-yellow-800">
      <strong>Loi belge :</strong> Art. 577-6 §6-8 du Code Civil - La majorité requise dépend de la nature de la résolution.
      Les travaux importants nécessitent généralement une majorité qualifiée (3/4 ou 4/5 des voix).
    </div>
  </div>

  <div class="flex justify-end gap-2 mt-4">
    <button
      type="button"
      on:click={() => dispatch('created', null)}
      class="px-4 py-2 text-sm text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
    >
      Annuler
    </button>
    <button
      type="submit"
      disabled={loading || !title.trim()}
      class="px-4 py-2 text-sm text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {loading ? 'Création...' : 'Créer la résolution'}
    </button>
  </div>
</form>
