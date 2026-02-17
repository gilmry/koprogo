<script lang="ts">
  import {
    energyCampaignsApi,
    type CreateCampaignDto,
    EnergyType,
  } from "../../lib/api/energy-campaigns";
  import BuildingSelector from "../BuildingSelector.svelte";

  export let organizationId: string;
  export let onCreated: ((campaign: any) => void) | undefined = undefined;
  export let onCancel: (() => void) | undefined = undefined;

  let selectedBuildingId = "";

  // Default deadline: 3 months from now
  const defaultDeadline = new Date();
  defaultDeadline.setMonth(defaultDeadline.getMonth() + 3);

  let formData: CreateCampaignDto = {
    building_id: undefined,
    campaign_name: "",
    deadline_participation: defaultDeadline.toISOString().split("T")[0],
    energy_types: [],
  };

  $: formData.building_id = selectedBuildingId || undefined;

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

  async function handleSubmit(e: Event) {
    e.preventDefault();
    loading = true;
    error = "";
    success = false;

    try {
      if (!formData.campaign_name.trim()) {
        throw new Error("Le nom de la campagne est obligatoire.");
      }
      if (formData.energy_types.length === 0) {
        throw new Error(
          "Selectionnez au moins un type d'energie (electricite, gaz ou chauffage).",
        );
      }
      if (!formData.deadline_participation) {
        throw new Error("La date limite de participation est obligatoire.");
      }
      const today = new Date().toISOString().split("T")[0];
      if (formData.deadline_participation <= today) {
        throw new Error(
          "La date limite doit etre dans le futur.",
        );
      }

      // Send as ISO datetime for backend DateTime<Utc>
      const payload = {
        ...formData,
        deadline_participation: new Date(formData.deadline_participation).toISOString(),
      };

      const campaign = await energyCampaignsApi.create(payload as any);
      success = true;
      if (onCreated) onCreated(campaign);

      setTimeout(() => {
        formData = {
          building_id: selectedBuildingId || undefined,
          campaign_name: "",
          deadline_participation: defaultDeadline.toISOString().split("T")[0],
          energy_types: [],
        };
        success = false;
      }, 2000);
    } catch (err: any) {
      error = err.message || "Erreur lors de la creation de la campagne";
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
    <!-- Building Selector -->
    <BuildingSelector bind:selectedBuildingId label="Immeuble concerné" required={false} />

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

    <!-- Deadline -->
    <div>
      <label
        for="deadline_participation"
        class="block text-sm font-medium text-gray-700"
      >
        Date limite de participation <span class="text-red-500">*</span>
      </label>
      <input
        type="date"
        id="deadline_participation"
        bind:value={formData.deadline_participation}
        required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
      />
      <p class="mt-1 text-xs text-gray-500">
        Date limite pour que les coproprietaires rejoignent la campagne. Recommandation : 3 a 6 mois.
      </p>
    </div>

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
        on:click={() => onCancel && onCancel()}
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
