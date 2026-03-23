<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
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
      error = $_('etatsDate.errors.selectBuildingAndUnit');
      return;
    }
    if (!notaryName || !notaryEmail) {
      error = $_('etatsDate.errors.notaryInfoRequired');
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
      error = err.message || $_('etatsDate.errors.creationFailed');
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
      <label for="building" class="block text-sm font-medium text-gray-700 mb-1">{$_('etatsDate.building')}</label>
      <select
        id="building"
        bind:value={buildingId}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
      >
        <option value="">-- {$_('common.select')} --</option>
        {#each buildings as building}
          <option value={building.id}>{building.name} - {building.address}</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="unit" class="block text-sm font-medium text-gray-700 mb-1">{$_('etatsDate.unit')}</label>
      <select
        id="unit"
        bind:value={unitId}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
        disabled={!buildingId}
      >
        <option value="">-- {$_('etatsDate.selectUnit')} --</option>
        {#each units as unit}
          <option value={unit.id}>{$_('etatsDate.unitLabel', { values: { number: unit.unit_number, floor: unit.floor ? `- ${$_('etatsDate.floor')} ${unit.floor}` : '' } })}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div>
      <label for="reference-date" class="block text-sm font-medium text-gray-700 mb-1">{$_('etatsDate.referenceDate')}</label>
      <input
        id="reference-date"
        type="date"
        bind:value={referenceDate}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        required
      />
    </div>

    <div>
      <label for="language" class="block text-sm font-medium text-gray-700 mb-1">{$_('etatsDate.documentLanguage')}</label>
      <select
        id="language"
        bind:value={language}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
      >
        <option value="fr">{$_('languages.fr')}</option>
        <option value="nl">{$_('languages.nl')}</option>
        <option value="de">{$_('languages.de')}</option>
      </select>
    </div>
  </div>

  <div class="border-t pt-4">
    <h4 class="text-sm font-semibold text-gray-900 mb-3">{$_('etatsDate.notaryInfo')}</h4>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div>
        <label for="notary-name" class="block text-sm font-medium text-gray-700 mb-1">{$_('common.name')}</label>
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
        <label for="notary-email" class="block text-sm font-medium text-gray-700 mb-1">{$_('common.email')}</label>
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
        <label for="notary-phone" class="block text-sm font-medium text-gray-700 mb-1">{$_('common.phone')}</label>
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
      {$_('common.cancel')}
    </button>
    <button
      type="submit"
      disabled={loading}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition disabled:opacity-50"
    >
      {loading ? $_('common.creating') : $_('etatsDate.createEtatDate')}
    </button>
  </div>
</form>
