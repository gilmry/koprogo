<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { budgetsApi, type CreateBudgetDto } from '../../lib/api/budgets';
  import { api } from '../../lib/api';
  import type { Building } from '../../lib/types';

  const dispatch = createEventDispatcher();

  let buildings: Building[] = [];
  let loading = false;
  let error = '';

  let buildingId = '';
  let fiscalYear = new Date().getFullYear();
  let ordinaryBudget = 0;
  let extraordinaryBudget = 0;
  let notes = '';

  $: totalBudget = ordinaryBudget + extraordinaryBudget;
  $: monthlyProvision = totalBudget > 0 ? totalBudget / 12 : 0;

  import { onMount } from 'svelte';

  onMount(async () => {
    try {
      const response = await api.get<{ data: Building[] }>('/buildings?page=1&per_page=100');
      buildings = response.data || [];
    } catch (err) {
      console.error('Error loading buildings:', err);
    }
  });

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  async function handleSubmit() {
    if (!buildingId) {
      error = 'Veuillez selectionner un immeuble';
      return;
    }
    if (totalBudget <= 0) {
      error = 'Le budget total doit etre superieur a 0';
      return;
    }

    try {
      loading = true;
      error = '';
      const data: CreateBudgetDto = {
        building_id: buildingId,
        fiscal_year: fiscalYear,
        ordinary_budget: ordinaryBudget,
        extraordinary_budget: extraordinaryBudget,
        notes: notes || undefined,
      };
      const budget = await budgetsApi.create(data);
      dispatch('created', budget);
    } catch (err: any) {
      error = err.message || 'Erreur lors de la creation du budget';
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

  <div>
    <label for="building" class="block text-sm font-medium text-gray-700 mb-1">Immeuble</label>
    <select
      id="building"
      bind:value={buildingId}
      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      required
    >
      <option value="">-- Selectionner --</option>
      {#each buildings as building}
        <option value={building.id}>{building.name} - {building.address}</option>
      {/each}
    </select>
  </div>

  <div>
    <label for="fiscal-year" class="block text-sm font-medium text-gray-700 mb-1">Annee fiscale</label>
    <input
      id="fiscal-year"
      type="number"
      min="2000"
      max="2100"
      bind:value={fiscalYear}
      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      required
    />
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div>
      <label for="ordinary" class="block text-sm font-medium text-gray-700 mb-1">
        Budget ordinaire (EUR)
      </label>
      <input
        id="ordinary"
        type="number"
        step="0.01"
        min="0"
        bind:value={ordinaryBudget}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        required
      />
      <p class="text-xs text-gray-500 mt-1">Charges courantes: entretien, assurances, nettoyage</p>
    </div>

    <div>
      <label for="extraordinary" class="block text-sm font-medium text-gray-700 mb-1">
        Budget extraordinaire (EUR)
      </label>
      <input
        id="extraordinary"
        type="number"
        step="0.01"
        min="0"
        bind:value={extraordinaryBudget}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        required
      />
      <p class="text-xs text-gray-500 mt-1">Gros travaux, renovations, projets speciaux</p>
    </div>
  </div>

  <!-- Summary -->
  <div class="bg-gray-50 rounded-lg p-4 space-y-2">
    <div class="flex justify-between text-sm">
      <span class="text-gray-600">Budget total</span>
      <span class="font-bold text-gray-900">{formatCurrency(totalBudget)}</span>
    </div>
    <div class="flex justify-between text-sm">
      <span class="text-gray-600">Provision mensuelle</span>
      <span class="font-medium text-primary-600">{formatCurrency(monthlyProvision)}</span>
    </div>
  </div>

  <div>
    <label for="notes" class="block text-sm font-medium text-gray-700 mb-1">Notes (optionnel)</label>
    <textarea
      id="notes"
      bind:value={notes}
      rows="3"
      class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      placeholder="Commentaires sur le budget..."
    ></textarea>
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
      {loading ? 'Creation...' : 'Creer le budget'}
    </button>
  </div>
</form>
