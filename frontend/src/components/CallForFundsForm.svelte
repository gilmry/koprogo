<script lang="ts">
  import { onMount } from 'svelte';
  import { api, callForFundsApi } from '../lib/api';
  import { toast } from '../stores/toast';

  export let buildingId: string | undefined = undefined;
  export let onSuccess: () => void = () => {};
  export let onCancel: () => void = () => {};

  let buildings: any[] = [];
  let loading = false;
  let submitting = false;

  // Form fields
  let selectedBuildingId = buildingId || '';
  let title = '';
  let description = '';
  let totalAmount = 0;
  let contributionType = 'regular';
  let callDate = new Date().toISOString().split('T')[0];
  let dueDate = '';
  let accountCode = '';

  onMount(async () => {
    await loadBuildings();
    // Set default due date to 30 days from call date
    const defaultDueDate = new Date();
    defaultDueDate.setDate(defaultDueDate.getDate() + 30);
    dueDate = defaultDueDate.toISOString().split('T')[0];
  });

  async function loadBuildings() {
    try {
      loading = true;
      buildings = await api.get('/buildings');
    } catch (error: any) {
      toast.error(error.message || 'Erreur lors du chargement des bâtiments');
    } finally {
      loading = false;
    }
  }

  async function handleSubmit(event: Event) {
    event.preventDefault();

    if (!selectedBuildingId) {
      toast.error('Veuillez sélectionner un bâtiment');
      return;
    }

    if (!title || !description) {
      toast.error('Veuillez remplir tous les champs obligatoires');
      return;
    }

    if (totalAmount <= 0) {
      toast.error('Le montant doit être supérieur à 0');
      return;
    }

    if (new Date(dueDate) < new Date(callDate)) {
      toast.error('La date d\'échéance doit être postérieure à la date d\'appel');
      return;
    }

    try {
      submitting = true;
      await callForFundsApi.create({
        building_id: selectedBuildingId,
        title,
        description,
        total_amount: totalAmount,
        contribution_type: contributionType,
        call_date: new Date(callDate).toISOString(),
        due_date: new Date(dueDate).toISOString(),
        account_code: accountCode || undefined,
      });

      toast.success('Appel de fonds créé avec succès');
      onSuccess();
    } catch (error: any) {
      toast.error(error.message || 'Erreur lors de la création');
    } finally {
      submitting = false;
    }
  }
</script>

<form on:submit={handleSubmit} class="space-y-6">
  <!-- Building Selection -->
  {#if !buildingId}
    <div>
      <label for="building" class="block text-sm font-medium text-gray-700">
        Bâtiment <span class="text-red-500">*</span>
      </label>
      <select
        id="building"
        bind:value={selectedBuildingId}
        required
        disabled={loading}
        class="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
      >
        <option value="">-- Sélectionner un bâtiment --</option>
        {#each buildings as building}
          <option value={building.id}>
            {building.name} - {building.address}
          </option>
        {/each}
      </select>
    </div>
  {/if}

  <!-- Title -->
  <div>
    <label for="title" class="block text-sm font-medium text-gray-700">
      Titre <span class="text-red-500">*</span>
    </label>
    <input
      type="text"
      id="title"
      bind:value={title}
      required
      placeholder="Ex: Charges du 1er trimestre 2025"
      class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
    />
  </div>

  <!-- Description -->
  <div>
    <label for="description" class="block text-sm font-medium text-gray-700">
      Description <span class="text-red-500">*</span>
    </label>
    <textarea
      id="description"
      bind:value={description}
      required
      rows="3"
      placeholder="Détails de l'appel de fonds..."
      class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
    ></textarea>
  </div>

  <!-- Contribution Type -->
  <div>
    <label for="contribution-type" class="block text-sm font-medium text-gray-700">
      Type de contribution <span class="text-red-500">*</span>
    </label>
    <select
      id="contribution-type"
      bind:value={contributionType}
      required
      class="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
    >
      <option value="regular">Charges régulières</option>
      <option value="extraordinary">Charges extraordinaires</option>
      <option value="advance">Avance</option>
      <option value="adjustment">Régularisation</option>
    </select>
    <p class="mt-1 text-sm text-gray-500">
      {#if contributionType === 'regular'}
        Charges courantes (ascenseur, chauffage, nettoyage...)
      {:else if contributionType === 'extraordinary'}
        Travaux exceptionnels (toiture, façade...)
      {:else if contributionType === 'advance'}
        Provision pour dépenses futures
      {:else if contributionType === 'adjustment'}
        Régularisation annuelle des charges
      {/if}
    </p>
  </div>

  <!-- Total Amount -->
  <div>
    <label for="total-amount" class="block text-sm font-medium text-gray-700">
      Montant total <span class="text-red-500">*</span>
    </label>
    <div class="mt-1 relative rounded-md shadow-sm">
      <input
        type="number"
        id="total-amount"
        bind:value={totalAmount}
        required
        min="0.01"
        step="0.01"
        placeholder="0.00"
        class="block w-full pr-12 border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
      />
      <div class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
        <span class="text-gray-500 sm:text-sm">EUR</span>
      </div>
    </div>
    <p class="mt-1 text-sm text-gray-500">
      Ce montant sera réparti entre les copropriétaires selon leurs tantièmes
    </p>
  </div>

  <!-- Call Date -->
  <div>
    <label for="call-date" class="block text-sm font-medium text-gray-700">
      Date d'appel <span class="text-red-500">*</span>
    </label>
    <input
      type="date"
      id="call-date"
      bind:value={callDate}
      required
      class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
    />
  </div>

  <!-- Due Date -->
  <div>
    <label for="due-date" class="block text-sm font-medium text-gray-700">
      Date d'échéance <span class="text-red-500">*</span>
    </label>
    <input
      type="date"
      id="due-date"
      bind:value={dueDate}
      required
      class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
    />
  </div>

  <!-- Account Code (Optional) -->
  <div>
    <label for="account-code" class="block text-sm font-medium text-gray-700">
      Code comptable PCMN (optionnel)
    </label>
    <input
      type="text"
      id="account-code"
      bind:value={accountCode}
      placeholder="Ex: 7000"
      class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
    />
    <p class="mt-1 text-sm text-gray-500">
      Compte comptable pour la comptabilisation (classe 7 - Produits)
    </p>
  </div>

  <!-- Info Box -->
  <div class="rounded-md bg-blue-50 p-4">
    <div class="flex">
      <div class="flex-shrink-0">
        <svg class="h-5 w-5 text-blue-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
      </div>
      <div class="ml-3 flex-1">
        <p class="text-sm text-blue-700">
          <strong>Note importante :</strong> Après création, l'appel de fonds sera en statut "Brouillon".
          Vous devrez l'envoyer manuellement pour générer les contributions individuelles pour chaque copropriétaire.
        </p>
      </div>
    </div>
  </div>

  <!-- Form Actions -->
  <div class="flex justify-end space-x-3 pt-4 border-t border-gray-200">
    <button
      type="button"
      on:click={onCancel}
      disabled={submitting}
      class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
    >
      Annuler
    </button>
    <button
      type="submit"
      disabled={submitting}
      class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
    >
      {submitting ? 'Création...' : 'Créer l\'appel de fonds'}
    </button>
  </div>
</form>
