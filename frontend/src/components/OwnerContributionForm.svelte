<script lang="ts">
  import { api } from '../lib/api';

  export let organizationId: string;
  export let onSuccess: () => void = () => {};

  let formData = {
    owner_id: '',
    unit_id: '',
    description: '',
    amount: '',
    contribution_type: 'regular',
    contribution_date: new Date().toISOString().split('T')[0],
    account_code: '7000'
  };

  let owners: any[] = [];
  let units: any[] = [];
  let loading = false;
  let error = '';

  // Load owners and units on mount
  async function loadData() {
    try {
      const [ownersData, unitsData] = await Promise.all([
        api.get('/owners'),
        api.get('/units')
      ]);
      owners = ownersData;
      units = unitsData;
    } catch (err: any) {
      error = err.message;
    }
  }

  // Load data on component mount
  $: if (organizationId) {
    loadData();
  }

  // Update account code when contribution type changes
  $: if (formData.contribution_type === 'regular') {
    formData.account_code = '7000';
  } else if (formData.contribution_type === 'extraordinary') {
    formData.account_code = '7100';
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = '';

    try {
      // Convert form data to API format
      const payload = {
        owner_id: formData.owner_id,
        unit_id: formData.unit_id || null,
        description: formData.description,
        amount: parseFloat(formData.amount),
        contribution_type: formData.contribution_type,
        contribution_date: new Date(formData.contribution_date).toISOString(),
        account_code: formData.account_code
      };

      await api.post('/owner-contributions', payload);

      // Reset form
      formData = {
        owner_id: '',
        unit_id: '',
        description: '',
        amount: '',
        contribution_type: 'regular',
        contribution_date: new Date().toISOString().split('T')[0],
        account_code: '7000'
      };

      onSuccess();
    } catch (err: any) {
      error = err.message || 'Erreur lors de la création de l\'appel de fonds';
    } finally {
      loading = false;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-semibold text-gray-900 mb-4">
    Nouvel Appel de Fonds
  </h3>

  {#if error}
    <div class="mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-4">
    <!-- Owner Selection -->
    <div>
      <label for="owner_id" class="block text-sm font-medium text-gray-700 mb-1">
        Copropriétaire *
      </label>
      <select
        id="owner_id"
        bind:value={formData.owner_id}
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="">Sélectionner un copropriétaire</option>
        {#each owners as owner}
          <option value={owner.id}>
            {owner.first_name} {owner.last_name}
          </option>
        {/each}
      </select>
    </div>

    <!-- Unit Selection (optional) -->
    <div>
      <label for="unit_id" class="block text-sm font-medium text-gray-700 mb-1">
        Lot (optionnel)
      </label>
      <select
        id="unit_id"
        bind:value={formData.unit_id}
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="">Aucun lot spécifique</option>
        {#each units as unit}
          <option value={unit.id}>
            Lot {unit.unit_number} - {unit.floor}
          </option>
        {/each}
      </select>
    </div>

    <!-- Contribution Type -->
    <div>
      <label for="contribution_type" class="block text-sm font-medium text-gray-700 mb-1">
        Type d'appel de fonds *
      </label>
      <select
        id="contribution_type"
        bind:value={formData.contribution_type}
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="regular">Charges courantes (7000)</option>
        <option value="extraordinary">Charges extraordinaires (7100)</option>
        <option value="advance">Avance</option>
        <option value="adjustment">Régularisation</option>
      </select>
    </div>

    <!-- Description -->
    <div>
      <label for="description" class="block text-sm font-medium text-gray-700 mb-1">
        Description *
      </label>
      <textarea
        id="description"
        bind:value={formData.description}
        required
        rows="3"
        placeholder="Ex: Appel de fonds T4 2025 - Charges courantes"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      ></textarea>
    </div>

    <!-- Amount -->
    <div>
      <label for="amount" class="block text-sm font-medium text-gray-700 mb-1">
        Montant (€) *
      </label>
      <input
        type="number"
        id="amount"
        bind:value={formData.amount}
        required
        min="0"
        step="0.01"
        placeholder="0.00"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>

    <!-- Contribution Date -->
    <div>
      <label for="contribution_date" class="block text-sm font-medium text-gray-700 mb-1">
        Date de l'appel *
      </label>
      <input
        type="date"
        id="contribution_date"
        bind:value={formData.contribution_date}
        required
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>

    <!-- Account Code (read-only, auto-filled) -->
    <div>
      <label for="account_code" class="block text-sm font-medium text-gray-700 mb-1">
        Code comptable PCMN
      </label>
      <input
        type="text"
        id="account_code"
        bind:value={formData.account_code}
        readonly
        class="w-full px-3 py-2 border border-gray-300 rounded-md bg-gray-50 text-gray-600"
      />
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3 pt-4">
      <button
        type="submit"
        disabled={loading}
        class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {loading ? 'Création...' : 'Créer l\'appel de fonds'}
      </button>
    </div>
  </form>
</div>
