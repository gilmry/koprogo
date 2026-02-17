<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    energyCampaignsApi,
    type CreateProviderOfferDto,
  } from "../../lib/api/energy-campaigns";

  export let campaignId: string;

  const dispatch = createEventDispatcher();

  let formData: CreateProviderOfferDto = {
    provider_name: "",
    price_kwh_electricity: undefined,
    price_kwh_gas: undefined,
    fixed_monthly_fee: 0,
    green_energy_pct: 0,
    contract_duration_months: 12,
    estimated_savings_pct: 0,
    offer_valid_until: "",
  };

  let loading = false;
  let error = "";
  let success = false;

  function setDefaultValidityDate() {
    const date = new Date();
    date.setMonth(date.getMonth() + 1); // Valid for 1 month by default
    formData.offer_valid_until = date.toISOString().split("T")[0];
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      // Validate
      if (!formData.provider_name.trim()) {
        throw new Error("Le nom du fournisseur est obligatoire");
      }
      if (!formData.price_kwh_electricity && !formData.price_kwh_gas) {
        throw new Error("Au moins un prix par kWh (electricite ou gaz) est obligatoire");
      }
      if (formData.contract_duration_months <= 0) {
        throw new Error("La duree du contrat doit etre superieure a 0");
      }
      if (!formData.offer_valid_until) {
        throw new Error("La date de validite est obligatoire");
      }
      if (formData.green_energy_pct < 0 || formData.green_energy_pct > 100) {
        throw new Error("Le pourcentage d'energie verte doit etre entre 0 et 100");
      }
      if (formData.estimated_savings_pct < 0 || formData.estimated_savings_pct > 100) {
        throw new Error("Le pourcentage d'economies doit etre entre 0 et 100");
      }

      // Send date as ISO datetime for backend DateTime<Utc>
      const payload = {
        ...formData,
        offer_valid_until: new Date(formData.offer_valid_until).toISOString(),
      };

      const offer = await energyCampaignsApi.addOffer(campaignId, payload as any);
      success = true;
      dispatch("created", offer);

      // Reset form after 2 seconds
      setTimeout(() => {
        formData = {
          provider_name: "",
          price_kwh_electricity: undefined,
          price_kwh_gas: undefined,
          fixed_monthly_fee: 0,
          green_energy_pct: 0,
          contract_duration_months: 12,
          estimated_savings_pct: 0,
          offer_valid_until: "",
        };
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || "Erreur lors de l'ajout de l'offre";
      console.error("Failed to create provider offer:", err);
    } finally {
      loading = false;
    }
  }

  // Initialize default validity date
  setDefaultValidityDate();
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <h3 class="text-lg font-medium text-gray-900 mb-4">
    💼 Ajouter une offre de fournisseur
  </h3>

  <p class="text-sm text-gray-600 mb-6">
    Enregistrez une offre reçue d'un fournisseur d'énergie pour permettre aux
    copropriétaires de la comparer.
  </p>

  {#if success}
    <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
      <p class="text-sm text-green-800">✅ Offre ajoutée avec succès !</p>
    </div>
  {/if}

  {#if error}
    <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
    </div>
  {/if}

  <form on:submit={handleSubmit} class="space-y-6">
    <!-- Provider Name -->
    <div>
      <label
        for="provider_name"
        class="block text-sm font-medium text-gray-700"
      >
        Nom du fournisseur <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="provider_name"
        bind:value={formData.provider_name}
        required
        placeholder="Ex: Engie, Luminus, TotalEnergies"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Pricing -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div>
        <label
          for="price_kwh_electricity"
          class="block text-sm font-medium text-gray-700"
        >
          Prix electricite (€/kWh)
        </label>
        <input
          type="number"
          id="price_kwh_electricity"
          bind:value={formData.price_kwh_electricity}
          min="0"
          step="0.0001"
          placeholder="0.1234"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
        <p class="mt-1 text-xs text-gray-500">
          Ex: 0.1234 €/kWh. Laisser vide si non concerne.
        </p>
      </div>
      <div>
        <label
          for="price_kwh_gas"
          class="block text-sm font-medium text-gray-700"
        >
          Prix gaz (€/kWh)
        </label>
        <input
          type="number"
          id="price_kwh_gas"
          bind:value={formData.price_kwh_gas}
          min="0"
          step="0.0001"
          placeholder="0.0567"
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
        />
        <p class="mt-1 text-xs text-gray-500">
          Ex: 0.0567 €/kWh. Laisser vide si non concerne.
        </p>
      </div>
    </div>

    <!-- Fixed Fee -->
    <div>
      <label
        for="fixed_fee"
        class="block text-sm font-medium text-gray-700"
      >
        Abonnement mensuel (€) <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="fixed_fee"
        bind:value={formData.fixed_monthly_fee}
        required
        min="0"
        step="0.01"
        placeholder="15.00"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
    </div>

    <!-- Contract Duration -->
    <div>
      <label
        for="contract_duration"
        class="block text-sm font-medium text-gray-700"
      >
        Durée du contrat (mois) <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="contract_duration"
        bind:value={formData.contract_duration_months}
        required
        min="1"
        max="60"
        placeholder="12"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">Généralement 12 ou 24 mois</p>
    </div>

    <!-- Green Energy Percentage -->
    <div>
      <label
        for="green_percentage"
        class="block text-sm font-medium text-gray-700"
      >
        Pourcentage d'energie verte (%) <span class="text-red-500">*</span>
      </label>
      <div class="flex items-center space-x-4">
        <input
          type="range"
          id="green_percentage"
          bind:value={formData.green_energy_pct}
          min="0"
          max="100"
          step="1"
          class="flex-1"
        />
        <span class="text-sm font-medium text-gray-900 w-12">
          {formData.green_energy_pct}%
        </span>
      </div>
      <p class="mt-1 text-xs text-gray-500">
        100% = Entierement renouvelable (eolien, solaire, hydraulique)
      </p>
    </div>

    <!-- Estimated Savings -->
    <div>
      <label for="savings" class="block text-sm font-medium text-gray-700">
        Economies estimees (%) <span class="text-red-500">*</span>
      </label>
      <input
        type="number"
        id="savings"
        bind:value={formData.estimated_savings_pct}
        required
        min="0"
        max="100"
        step="0.1"
        placeholder="15"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        Pourcentage d'economies par rapport au tarif actuel moyen
      </p>
    </div>

    <!-- Validity Date -->
    <div>
      <label
        for="valid_until"
        class="block text-sm font-medium text-gray-700"
      >
        Valide jusqu'au <span class="text-red-500">*</span>
      </label>
      <input
        type="date"
        id="valid_until"
        bind:value={formData.offer_valid_until}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
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
          Ajout en cours...
        {:else}
          ✅ Ajouter l'offre
        {/if}
      </button>
    </div>
  </form>
</div>
