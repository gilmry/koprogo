<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { etatsDatesApi, EtatDateLanguage, type CreateEtatDateDto } from '../../lib/api/etats-dates';
  import { api } from '../../lib/api';
  import type { Building } from '../../lib/types';
  import { withErrorHandling } from "../../lib/utils/error.utils";
  // Track H Story H2 — validate-before-compute (FR-H2). Banner + toast 422.
  import ConformityBanner from '../../lib/components/shared/ConformityBanner.svelte';
  import {
    buildConformityStatus,
    showConformityToast,
  } from '../../lib/utils/conformity';

  const dispatch = createEventDispatcher();

  let buildings: Building[] = [];
  let selectedBuilding: Building | null = null;
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
    const result = await withErrorHandling({
      action: () => api.get<{ data: Building[] }>('/buildings?page=1&per_page=100'),
    });
    if (result) buildings = result.data || [];
  });

  async function loadUnits() {
    if (!buildingId) {
      units = [];
      unitId = '';
      selectedBuilding = null;
      return;
    }
    const result = await withErrorHandling({
      action: () => api.get(`/buildings/${buildingId}/units`),
    });
    units = result || [];
    unitId = '';
    // Track H Story H2 — récupère le building enrichi (is_conformant + metrics).
    // `GET /buildings` (liste paginée) renvoie toujours des métriques VIDES
    // par défaut (units_count:0, is_conformant:false — choix de perf pour
    // éviter un JOIN sur la pagination, cf. building_use_cases.rs
    // `to_response_dto` / `BuildingMetrics::empty()`), jamais `undefined` —
    // un check `=== undefined` ne se déclenche donc jamais et le formulaire
    // affichait "non conforme" pour tout immeuble réellement conforme.
    // Seul `GET /buildings/{id}` calcule les vraies métriques : on le
    // recharge systématiquement pour le building sélectionné.
    selectedBuilding =
      buildings.find((b) => b.id === buildingId) || null;
    try {
      selectedBuilding = await api.get<Building>(`/buildings/${buildingId}`);
    } catch (e) {
      console.error('Failed to load building metrics:', e);
    }
  }

  $: if (buildingId) loadUnits();

  // Track H Story H2 — Statut conformité dérivé.
  $: conformityStatus =
    selectedBuilding && selectedBuilding.is_conformant !== undefined
      ? buildConformityStatus({
          is_conformant: !!selectedBuilding.is_conformant,
          total_units: selectedBuilding.total_units,
          units_count: selectedBuilding.units_count ?? 0,
          total_tantiemes: selectedBuilding.total_tantiemes,
          quota_delta: selectedBuilding.quota_delta ?? '0',
        })
      : null;
  $: canCompute = conformityStatus ? conformityStatus.is_conformant : true;

  async function handleSubmit() {
    if (!buildingId || !unitId) {
      error = $_('etatsDate.errors.selectBuildingAndUnit');
      return;
    }
    if (!notaryName || !notaryEmail) {
      error = $_('etatsDate.errors.notaryInfoRequired');
      return;
    }

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
    // Track H Story H2 — try/catch direct pour intercepter le 422
    // BUILDING_NOT_CONFORMANT et afficher le toast narratif.
    loading = true;
    try {
      const result = await etatsDatesApi.create(data);
      dispatch('created', result);
    } catch (err) {
      if (!showConformityToast(err)) {
        error = $_('etatsDate.errors.creationFailed');
      }
    } finally {
      loading = false;
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-6" data-testid="etat-date-create-form">
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-3">
      <p class="text-sm text-red-700">{error}</p>
    </div>
  {/if}

  <!-- Track H Story H2 — Banner conformité (FR-H2). DOM-absent si conformant
       ou si aucun building sélectionné. -->
  {#if conformityStatus && selectedBuilding}
    <ConformityBanner
      status={conformityStatus}
      buildingId={selectedBuilding.id}
      buildingName={selectedBuilding.name}
    />
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
      disabled={loading || !canCompute}
      aria-disabled={loading || !canCompute}
      title={!canCompute ? $_('conformity.toast_title') : ''}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition disabled:opacity-50 disabled:cursor-not-allowed"
      data-testid="etat-date-generate-button"
      data-can-compute={canCompute}
    >
      {loading ? $_('common.creating') : $_('etatsDate.createEtatDate')}
    </button>
  </div>
</form>
