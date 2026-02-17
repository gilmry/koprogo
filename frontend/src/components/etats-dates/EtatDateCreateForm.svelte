<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { etatsDatesApi, EtatDateLanguage, type CreateEtatDateDto } from '../../lib/api/etats-dates';
  import { api } from '../../lib/api';
  import type { Building } from '../../lib/types';

  const dispatch = createEventDispatcher();

  let buildings: Building[] = [];
  let units: any[] = [];
  let loading = false;
  let error = '';

  let buildingId = '';
  let unitId = '';
  let referenceDate = new Date().toISOString().split('T')[0];
  let language: EtatDateLanguage = EtatDateLanguage.Fr;
  let notaryName = '';
  let notaryEmail = '';
  let notaryPhone = '';

  onMount(async () => {
    try {
      const response = await api.get<{ data: Building[] }>('/buildings?page=1&per_page=100');
      buildings = response.data || [];
    } catch (err) {
      console.error('Error loading buildings:', err);
    }
  });

  async function loadUnits() {
    if (!buildingId) {
      units = [];
      unitId = '';
      return;
    }
    try {
      units = await api.get(`/buildings/${buildingId}/units`);
      unitId = '';
    } catch (err) {
      console.error('Error loading units:', err);
      units = [];
    }
  }

  $: if (buildingId) loadUnits();

  async function handleSubmit() {
    if (!buildingId || !unitId) {
      error = 'Veuillez selectionner un immeuble et un lot';
      return;
    }
    if (!notaryName || !notaryEmail) {
      error = 'Le nom et l\'email du notaire sont obligatoires';
      return;
    }

    try {
      loading = true;
      error = '';
      const data: CreateEtatDateDto = {
        building_id: buildingId,
        unit_id: unitId,
        reference_date: new Date(referenceDate).toISOString(),
        language,
        notary_name: notaryName,
        notary_email: notaryEmail,
        notary_phone: notaryPhone || undefined,
      };
      const etatDate = await etatsDatesApi.create(data);
      dispatch('created', etatDate);
    } catch (err: any) {
      error = err.message || 'Erreur lors de la creation';
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-6">
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-3">
      <p class="text-sm text-red-700">{error}</p>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div>
      <label for="building" class="block text-sm font-medium text-gray-700 mb-1">Immeuble</label>
      <select
        id="building"
        bind:value={buildingId}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
      >
        <option value="">-- Selectionner --</option>
        {#each buildings as building}
          <option value={building.id}>{building.name} - {building.address}</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="unit" class="block text-sm font-medium text-gray-700 mb-1">Lot</label>
      <select
        id="unit"
        bind:value={unitId}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
        disabled={!buildingId}
      >
        <option value="">-- Selectionner un lot --</option>
        {#each units as unit}
          <option value={unit.id}>Lot {unit.unit_number} {unit.floor ? `- Etage ${unit.floor}` : ''}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div>
      <label for="reference-date" class="block text-sm font-medium text-gray-700 mb-1">Date de reference</label>
      <input
        id="reference-date"
        type="date"
        bind:value={referenceDate}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
      />
    </div>

    <div>
      <label for="language" class="block text-sm font-medium text-gray-700 mb-1">Langue du document</label>
      <select
        id="language"
        bind:value={language}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
      >
        <option value="fr">Francais</option>
        <option value="nl">Neerlandais</option>
        <option value="de">Allemand</option>
      </select>
    </div>
  </div>

  <div class="border-t pt-4">
    <h4 class="text-sm font-semibold text-gray-900 mb-3">Informations du Notaire</h4>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div>
        <label for="notary-name" class="block text-sm font-medium text-gray-700 mb-1">Nom</label>
        <input
          id="notary-name"
          type="text"
          bind:value={notaryName}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          placeholder="Maitre Dupont"
          required
        />
      </div>
      <div>
        <label for="notary-email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
        <input
          id="notary-email"
          type="email"
          bind:value={notaryEmail}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          placeholder="notaire@example.be"
          required
        />
      </div>
      <div>
        <label for="notary-phone" class="block text-sm font-medium text-gray-700 mb-1">Telephone (optionnel)</label>
        <input
          id="notary-phone"
          type="tel"
          bind:value={notaryPhone}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          placeholder="+32 2 123 45 67"
        />
      </div>
    </div>
  </div>

  <div class="flex justify-end space-x-3">
    <button
      type="button"
      on:click={() => dispatch('cancel')}
      class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition"
    >
      Annuler
    </button>
    <button
      type="submit"
      disabled={loading}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition disabled:opacity-50"
    >
      {loading ? 'Creation...' : 'Creer l\'etat date'}
    </button>
  </div>
</form>
