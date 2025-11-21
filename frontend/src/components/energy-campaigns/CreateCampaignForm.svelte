<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    energyCampaignsApi,
    type CreateCampaignDto,
    EnergyType,
  } from "../../lib/api/energy-campaigns";

  export let organizationId: string;
  export let buildingId: string | undefined = undefined;

  const dispatch = createEventDispatcher();

  let formData: CreateCampaignDto = {
    organization_id: organizationId,
    building_id: buildingId,
    campaign_name: "",
    energy_types: [],
    campaign_start_date: new Date().toISOString().split("T")[0],
    campaign_end_date: "",
  };

  let loading = false;
  let error = "";
  let success = false;

  function toggleEnergyType(type: EnergyType) {
    if (formData.energy_types.includes(type)) {
      formData.energy_types = formData.energy_types.filter((t) => t !== type);
    } else {
      formData.energy_types = [...formData.energy_types, type];
    }
  }

  function setDefaultEndDate() {
    if (formData.campaign_start_date) {
      const startDate = new Date(formData.campaign_start_date);
      startDate.setMonth(startDate.getMonth() + 3); // 3 months campaign by default
      formData.campaign_end_date = startDate.toISOString().split("T")[0];
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      // Validate
      if (!formData.campaign_name.trim()) {
        throw new Error("Le nom de la campagne est obligatoire");
      }
      if (formData.energy_types.length === 0) {
        throw new Error(
          "Sélectionnez au moins un type d'énergie (électricité, gaz ou chauffage)",
        );
      }
      if (!formData.campaign_start_date || !formData.campaign_end_date) {
        throw new Error("Les dates de début et de fin sont obligatoires");
      }
      if (formData.campaign_end_date <= formData.campaign_start_date) {
        throw new Error(
          "La date de fin doit être postérieure à la date de début",
        );
      }

      const campaign = await energyCampaignsApi.create(formData);
      success = true;
      dispatch("created", campaign);

      // Reset form
      setTimeout(() => {
        formData = {
          organization_id: organizationId,
          building_id: buildingId,
          campaign_name: "",
          energy_types: [],
          campaign_start_date: new Date().toISOString().split("T")[0],
          campaign_end_date: "",
        };
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || "Erreur lors de la création de la campagne";
      console.error("Failed to create campaign:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    ➕ Créer une campagne d'achat groupé d'énergie
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    Négociez collectivement vos contrats d'énergie avec d'autres copropriétés.
    <strong>Économies attendues: 15-25%</strong> sur vos factures d'électricité
    et de gaz.
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">
        ✅ Campagne créée avec succès ! Vous allez être redirigé...
      </p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Campaign Name -->
    <div>
      <label
        for="campaign_name"
        class="block text-sm font-medium text-gray-700"
      >
        Nom de la campagne <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="campaign_name"
        bind:value={formData.campaign_name}
        required
        placeholder="Ex: Achat groupé électricité 2025"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        Un nom clair qui identifie votre campagne d'achat groupé.
      </p>
    </div>

    <!-- Energy Types -->
    <div>
      <label class="block text-sm font-medium text-gray-700 mb-2">
        Types d'énergie <span class="text-red-500">*</span>
      </label>
      <div class="space-y-2">
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Electricity)}
            on:change={() => toggleEnergyType(EnergyType.Electricity)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            ⚡ Électricité (recommandé - économies jusqu'à 25%)
          </span>
        </label>
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Gas)}
            on:change={() => toggleEnergyType(EnergyType.Gas)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            🔥 Gaz (recommandé - économies jusqu'à 20%)
          </span>
        </label>
        <label class="flex items-center">
          <input
            type="checkbox"
            checked={formData.energy_types.includes(EnergyType.Heating)}
            on:change={() => toggleEnergyType(EnergyType.Heating)}
            class="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
          />
          <span class="ml-2 text-sm text-gray-700">
            🌡️ Chauffage (mazout, pellets, etc.)
          </span>
        </label>
      </div>
      <p class="mt-2 text-xs text-gray-500">
        Sélectionnez tous les types d'énergie que vous souhaitez négocier.
      </p>
    </div>

    <!-- Dates -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="start_date"
          class="block text-sm font-medium text-gray-700"
        >
          Date de début <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="start_date"
          bind:value={formData.campaign_start_date}
          on:change={setDefaultEndDate}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
      <div>
        <label for="end_date" class="block text-sm font-medium text-gray-700">
          Date de fin <span class="text-red-500">*</span>
        </label>
        <input
          type="date"
          id="end_date"
          bind:value={formData.campaign_end_date}
          required
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
      </div>
    </div>
    <p class="text-xs text-gray-500">
      Recommandation: 3-6 mois pour collecter les factures et négocier avec les
      fournisseurs.
    </p>

    <!-- GDPR Notice -->
    <div class="p-4 bg-blue-50 border border-blue-200 rounded-md">
      <h4 class="text-sm font-medium text-blue-900 mb-2">
        🔒 Protection des données GDPR
      </h4>
      <ul class="text-xs text-blue-800 space-y-1">
        <li>
          ✅ Toutes les factures sont chiffrées avec AES-256-GCM
        </li>
        <li>
          ✅ K-anonymité garantie: minimum 5 participants avant publication
        </li>
        <li>
          ✅ Consentement explicite requis pour chaque upload
        </li>
        <li>✅ Droit à l'oubli: suppression à tout moment</li>
        <li>
          ✅ Conforme CREG (Commission de Régulation de l'Électricité et du
          Gaz)
        </li>
      </ul>
    </div>

    <!-- Submit Button -->
    <div class="flex justify-end space-x-3">
      <button
        type="button"
        on:click={() => dispatch("cancel")}
        class="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
      >
        Annuler
      </button>
      <button
        type="submit"
        disabled={loading}
        class="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-block animate-spin mr-2">⏳</span>
          Création en cours...
        {:else}
          ✅ Créer la campagne
        {/if}
      </button>
    </div>
  </form>
</div>
