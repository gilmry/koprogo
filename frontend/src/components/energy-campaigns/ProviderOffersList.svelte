<script lang="ts">
  import { onMount } from "svelte";
  import {
    energyCampaignsApi,
    type ProviderOffer,
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

      // Calculate best offer (highest savings percentage)
      if (offers.length > 0) {
        bestOffer = offers.reduce((best, current) => {
          return current.estimated_savings_pct > best.estimated_savings_pct
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

  function formatPrice(euros: number): string {
    return euros.toFixed(4) + " €";
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE");
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

  function getGreenScoreLabel(score: number): string {
    if (score >= 10) return "Vert";
    if (score >= 5) return "Mixte";
    return "Classique";
  }
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
                Score vert: {getGreenScoreLabel(offer.green_score)} ({offer.green_score}/10)
              </p>
            </div>
            {#if offer.id === selectedOfferId}
              <span class="text-green-600 text-xl" title="Offre sélectionnée">
                ✅
              </span>
            {:else if offer.id === bestOffer?.id}
              <span
                class="text-indigo-600 text-xl"
                title="Meilleure offre (economies)"
              >
                ⭐
              </span>
            {/if}
          </div>

          <!-- Price -->
          <div class="mb-3">
            {#if offer.price_kwh_electricity != null}
              <div class="text-lg font-bold text-gray-900">
                ⚡ {formatPrice(offer.price_kwh_electricity)}/kWh
              </div>
            {/if}
            {#if offer.price_kwh_gas != null}
              <div class="text-lg font-bold text-gray-900">
                🔥 {formatPrice(offer.price_kwh_gas)}/kWh
              </div>
            {/if}
            <div class="text-sm text-gray-600 mt-1">
              + {offer.fixed_monthly_fee.toFixed(2)} €/mois
            </div>
          </div>

          <!-- Green Energy Badge -->
          <div class="mb-3">
            {#if offer.green_energy_pct > 0}
              {@const badge = getGreenBadge(offer.green_energy_pct)}
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

          <!-- Estimated Savings -->
          <div class="text-sm font-medium text-green-600 mb-2">
            💸 Économies estimées: {offer.estimated_savings_pct.toFixed(1)}%
          </div>

          <!-- Valid Until -->
          <div class="text-xs text-gray-500 mb-3">
            Valide jusqu'au {formatDate(offer.offer_valid_until)}
          </div>

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
                  Économies
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
                    {offer.price_kwh_electricity != null ? formatPrice(offer.price_kwh_electricity) : "-"}
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.fixed_monthly_fee.toFixed(2)} €/mois
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.contract_duration_months} mois
                  </td>
                  <td class="px-4 py-2 text-sm text-gray-900">
                    {offer.green_energy_pct}%
                  </td>
                  <td class="px-4 py-2 text-sm font-medium text-green-600">
                    {offer.estimated_savings_pct.toFixed(1)}%
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
