<script lang="ts">
  import { onMount } from "svelte";
  import {
    energyCampaignsApi,
    type ProviderOffer,
    EnergyType,
  } from "../../lib/api/energy-campaigns";

  export let campaignId: string;
  export let selectedOfferId: string | undefined = undefined;
  export let canSelect = false; // Admin/Syndic can select offer

  let offers: ProviderOffer[] = [];
  let loading = true;
  let error = "";
  let bestOffer: ProviderOffer | null = null;

  onMount(async () => {
    await loadOffers();
  });

  async function loadOffers() {
    try {
      loading = true;
      error = "";
      offers = await energyCampaignsApi.listOffers(campaignId);

      // Calculate best offer (lowest price)
      if (offers.length > 0) {
        bestOffer = offers.reduce((best, current) => {
          return current.price_per_kwh_cents < best.price_per_kwh_cents
            ? current
            : best;
        });
      }
    } catch (err: any) {
      error = err.message || "Erreur lors du chargement des offres";
      console.error("Failed to load offers:", err);
    } finally {
      loading = false;
    }
  }

  function formatPrice(cents: number): string {
    return (cents / 100).toFixed(4) + " €";
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE");
  }

  function getEnergyTypeLabel(type: EnergyType): string {
    const labels = {
      [EnergyType.Electricity]: "⚡ Électricité",
      [EnergyType.Gas]: "🔥 Gaz",
      [EnergyType.Heating]: "🌡️ Chauffage",
    };
    return labels[type] || type;
  }

  function getGreenBadge(percentage: number): {
    color: string;
    label: string;
    icon: string;
  } {
    if (percentage >= 80) {
      return {
        color: "bg-green-100 text-green-800",
        label: `${percentage}% vert`,
        icon: "🌱",
      };
    } else if (percentage >= 50) {
      return {
        color: "bg-yellow-100 text-yellow-800",
        label: `${percentage}% vert`,
        icon: "🌿",
      };
    } else {
      return {
        color: "bg-gray-100 text-gray-800",
        label: `${percentage}% vert`,
        icon: "⚡",
      };
    }
  }

  function calculateAnnualCost(offer: ProviderOffer, annualKwh: number): number {
    const energyCost = (offer.price_per_kwh_cents * annualKwh) / 100;
    const fixedCost = offer.fixed_monthly_fee_cents
      ? (offer.fixed_monthly_fee_cents * 12) / 100
      : 0;
    return energyCost + fixedCost;
  }

  // Example: Assume 3500 kWh/year per household
  const estimatedAnnualKwh = 3500;
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <h3 class="text-lg leading-6 font-medium text-gray-900">
      💼 Offres des fournisseurs d'énergie
    </h3>
    <p class="mt-1 text-sm text-gray-500">
      Comparez les offres reçues pour sélectionner la meilleure option.
    </p>
  </div>

  {#if loading}
    <div class="p-8 text-center">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"
      ></div>
      <p class="mt-2 text-sm text-gray-500">Chargement des offres...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
      <button
        on:click={loadOffers}
        class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
      >
        Réessayer
      </button>
    </div>
  {:else if offers.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucune offre reçue pour le moment</p>
      <p class="mt-2 text-sm text-gray-400">
        Les fournisseurs soumettront leurs offres pendant la phase de
        négociation.
      </p>
    </div>
  {:else}
    <!-- Offers Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-4">
      {#each offers as offer}
        <div
          class="border rounded-lg p-4 hover:shadow-lg transition-shadow {offer.id ===
          selectedOfferId
            ? 'border-green-500 bg-green-50'
            : offer.id === bestOffer?.id
              ? 'border-indigo-500 bg-indigo-50'
              : 'border-gray-200'}"
        >
          <!-- Header -->
          <div class="flex items-start justify-between mb-3">
            <div>
              <h4 class="font-semibold text-gray-900">{offer.provider_name}</h4>
              <p class="text-xs text-gray-500">
                {getEnergyTypeLabel(offer.energy_type)}
              </p>
            </div>
            {#if offer.id === selectedOfferId}
              <span class="text-green-600 text-xl" title="Offre sélectionnée">
                ✅
              </span>
            {:else if offer.id === bestOffer?.id}
              <span
                class="text-indigo-600 text-xl"
                title="Meilleure offre (prix)"
              >
                ⭐
              </span>
            {/if}
          </div>

          <!-- Price -->
          <div class="mb-3">
            <div class="text-2xl font-bold text-gray-900">
              {formatPrice(offer.price_per_kwh_cents)}
            </div>
            <div class="text-xs text-gray-500">par kWh</div>
            {#if offer.fixed_monthly_fee_cents}
              <div class="text-sm text-gray-600 mt-1">
                + {formatPrice(offer.fixed_monthly_fee_cents)}/mois
              </div>
            {/if}
          </div>

          <!-- Green Energy Badge -->
          <div class="mb-3">
            {#if offer.green_energy_percentage > 0}
              {@const badge = getGreenBadge(offer.green_energy_percentage)}
              <span
                class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.color}"
              >
                <span class="mr-1">{badge.icon}</span>
                {badge.label}
              </span>
            {:else}
              <span
                class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800"
              >
                ⚡ Énergie classique
              </span>
            {/if}
          </div>

          <!-- Contract Duration -->
          <div class="text-sm text-gray-700 mb-2">
            📅 Durée: {offer.contract_duration_months} mois
          </div>

          <!-- Estimated Annual Cost -->
          <div class="text-sm text-gray-700 mb-2">
            💰 Coût annuel estimé (3500 kWh):
            <strong>
              {calculateAnnualCost(offer, estimatedAnnualKwh).toFixed(2)} €
            </strong>
          </div>

          <!-- Estimated Savings -->
          {#if offer.estimated_annual_savings_cents}
            <div class="text-sm font-medium text-green-600 mb-2">
              💸 Économies estimées:
              {formatPrice(offer.estimated_annual_savings_cents)}/an
            </div>
          {/if}

          <!-- Valid Until -->
          <div class="text-xs text-gray-500 mb-3">
            Valide jusqu'au {formatDate(offer.offer_valid_until)}
          </div>

          <!-- Terms & Conditions -->
          {#if offer.terms_and_conditions_url}
            <a
              href={offer.terms_and_conditions_url}
              target="_blank"
              rel="noopener noreferrer"
              class="text-xs text-indigo-600 hover:text-indigo-800 underline"
            >
              📄 Conditions générales
            </a>
          {/if}

          <!-- Selection Button (Admin only) -->
          {#if canSelect && !selectedOfferId}
            <button
              on:click={() => {
                /* Dispatch select event */
              }}
              class="mt-3 w-full px-3 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700"
            >
              Sélectionner cette offre
            </button>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Comparison Table (if more than 2 offers) -->
    {#if offers.length >= 2}
      <div class="p-4 bg-gray-50 border-t border-gray-200">
        <h4 class="text-sm font-medium text-gray-900 mb-3">
          📊 Tableau comparatif
        </h4>
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-100">
              <tr>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Fournisseur
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Prix/kWh
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Coût fixe
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Durée
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Vert
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase"
                >
                  Coût annuel
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              {#each offers as offer}
                <tr
                  class="hover:bg-gray-50 {offer.id === bestOffer?.id
                    ? 'bg-indigo-50'
                    : ''}"
                >
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.provider_name}
                    {#if offer.id === bestOffer?.id}
                      <span class="ml-1">⭐</span>
                    {/if}
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {formatPrice(offer.price_per_kwh_cents)}
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.fixed_monthly_fee_cents
                      ? formatPrice(offer.fixed_monthly_fee_cents) + "/mois"
                      : "-"}
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.contract_duration_months} mois
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.green_energy_percentage}%
                  </td>
                  <td class="px-4 py-2 text-sm font-medium text-gray-900">
                    {calculateAnnualCost(offer, estimatedAnnualKwh).toFixed(
                      2,
                    )} €
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  {/if}
</div>
