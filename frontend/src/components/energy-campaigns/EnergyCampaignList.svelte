<script lang="ts">
  import { onMount } from "svelte";
  import {
    energyCampaignsApi,
    type EnergyCampaign,
    CampaignStatus,
  } from "../../lib/api/energy-campaigns";
  import CampaignStatusBadge from "./CampaignStatusBadge.svelte";

  export let organizationId: string | undefined = undefined;

  let campaigns: EnergyCampaign[] = [];
  let loading = true;
  let error = "";

  onMount(async () => {
    await loadCampaigns();
  });

  async function loadCampaigns() {
    try {
      loading = true;
      error = "";
      campaigns = await energyCampaignsApi.list(organizationId);
    } catch (err: any) {
      error =
        err.message || "Erreur lors du chargement des campagnes d'énergie";
      console.error("Failed to load energy campaigns:", err);
    } finally {
      loading = false;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE");
  }

  function getEnergyTypesLabel(types: string[]): string {
    const labels: Record<string, string> = {
      Electricity: "⚡ Électricité",
      Gas: "🔥 Gaz",
      Heating: "🌡️ Chauffage",
    };
    return types.map((t) => labels[t] || t).join(", ");
  }

  function getCampaignProgress(campaign: EnergyCampaign): number {
    const statuses = [
      CampaignStatus.Draft,
      CampaignStatus.CollectingData,
      CampaignStatus.Negotiating,
      CampaignStatus.AwaitingFinalVote,
      CampaignStatus.Finalized,
      CampaignStatus.Completed,
    ];
    const currentIndex = statuses.indexOf(campaign.status);
    return ((currentIndex + 1) / statuses.length) * 100;
  }
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        📊 Achats Groupés d'Énergie
      </h3>
      <a
        href="/energy-campaigns/new"
        class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
      >
        <span class="mr-2">➕</span>
        Nouvelle campagne
      </a>
    </div>
    <p class="mt-1 text-sm text-gray-500">
      Négociez collectivement vos contrats d'énergie. Économies de 15-25% sur
      vos factures.
    </p>
  </div>

  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
      <p class="mt-2 text-sm text-gray-500">Chargement des campagnes...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
      <button
        on:click={loadCampaigns}
        class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
      >
        Réessayer
      </button>
    </div>
  {:else if campaigns.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucune campagne d'achat groupé d'énergie</p>
      <p class="mt-2 text-sm text-gray-400">
        Créez votre première campagne pour commencer à économiser sur vos
        factures d'énergie.
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each campaigns as campaign}
        <li class="hover:bg-gray-50">
          <a
            href="/energy-campaigns/detail?id={campaign.id}"
            class="block px-4 py-4 sm:px-6"
          >
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-3">
                  <h4 class="text-sm font-medium text-indigo-600 truncate">
                    {campaign.campaign_name}
                  </h4>
                  <CampaignStatusBadge status={campaign.status} />
                </div>
                <div class="mt-2 flex items-center text-sm text-gray-500">
                  <span>{getEnergyTypesLabel(campaign.energy_types)}</span>
                  <span class="mx-2">•</span>
                  <span>👥 {campaign.total_participants} participants</span>
                  {#if campaign.total_kwh_electricity}
                    <span class="mx-2">•</span>
                    <span>
                      ⚡ {campaign.total_kwh_electricity.toLocaleString()} kWh
                    </span>
                  {/if}
                </div>
                <div class="mt-2 flex items-center text-xs text-gray-400">
                  <span>
                    {formatDate(campaign.campaign_start_date)} →
                    {formatDate(campaign.campaign_end_date)}
                  </span>
                  {#if campaign.offers_received.length > 0}
                    <span class="mx-2">•</span>
                    <span>
                      💼 {campaign.offers_received.length} offres reçues
                    </span>
                  {/if}
                </div>
              </div>
              <div class="ml-4">
                <svg
                  class="h-5 w-5 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 5l7 7-7 7"
                  />
                </svg>
              </div>
            </div>
            <!-- Progress bar -->
            <div class="mt-3">
              <div class="w-full bg-gray-200 rounded-full h-1.5">
                <div
                  class="bg-indigo-600 h-1.5 rounded-full transition-all duration-300"
                  style="width: {getCampaignProgress(campaign)}%"
                ></div>
              </div>
            </div>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</div>
