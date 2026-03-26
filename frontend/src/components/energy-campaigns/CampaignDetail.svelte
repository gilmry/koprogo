<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    energyCampaignsApi,
    energyBillsApi,
    type EnergyCampaign,
    type CampaignStatistics,
    type EnergyBillUpload as EnergyBillUploadType,
    CampaignStatus,
  } from "../../lib/api/energy-campaigns";
  import CampaignStatusBadge from "./CampaignStatusBadge.svelte";
  import { toast } from "../../stores/toast";
  import ProviderOffersList from "./ProviderOffersList.svelte";
  import EnergyBillUpload from "./EnergyBillUpload.svelte";
  import { formatDateShort } from "../../lib/utils/date.utils";
  import { withLoadingState, withErrorHandling } from "../../lib/utils/error.utils";

  export let campaignId: string;
  export let currentUserId: string;
  export let currentUnitId: string | undefined = undefined;
  export let isAdmin = false;

  let campaign: EnergyCampaign | null = null;
  let stats: CampaignStatistics | null = null;
  let myUploads: EnergyBillUploadType[] = [];
  let loading = true;
  let error = "";
  let showUploadForm = false;

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    await withLoadingState({
      action: async () => {
        const [campaignData, statsData] = await Promise.all([
          energyCampaignsApi.getById(campaignId),
          energyCampaignsApi.getStats(campaignId),
        ]);
        let uploads: EnergyBillUploadType[] = [];
        if (currentUnitId) {
          const allUploads = await energyBillsApi.getMyUploads();
          uploads = allUploads.filter(
            (u) => u.campaign_id === campaignId && !u.deleted_at,
          );
        }
        return { campaignData, statsData, uploads };
      },
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: ({ campaignData, statsData, uploads }) => {
        campaign = campaignData;
        stats = statsData;
        myUploads = uploads;
      },
      errorMessage: $_("energy.campaign.loadError"),
    });
  }

  function getProgressPercentage(): number {
    if (!campaign) return 0;
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

  function getStatusDescription(): string {
    if (!campaign) return "";
    switch (campaign.status) {
      case CampaignStatus.Draft:
        return $_("energy.campaign.statusDraft");
      case CampaignStatus.CollectingData:
        return $_("energy.campaign.statusCollectingData");
      case CampaignStatus.Negotiating:
        return $_("energy.campaign.statusNegotiating");
      case CampaignStatus.AwaitingFinalVote:
        return $_("energy.campaign.statusAwaitingFinalVote");
      case CampaignStatus.Finalized:
        return $_("energy.campaign.statusFinalized");
      case CampaignStatus.Completed:
        return $_("energy.campaign.statusCompleted");
      default:
        return "";
    }
  }

  function canUpload(): boolean {
    return (
      campaign !== null &&
      currentUnitId !== undefined &&
      (campaign.status === CampaignStatus.CollectingData ||
        campaign.status === CampaignStatus.Negotiating)
    );
  }

  async function handleUploadComplete() {
    showUploadForm = false;
    await loadData();
  }

  async function withdrawConsent(uploadId: string) {
    if (!confirm($_("energy.withdrawConsentConfirm"))) return;
    await withErrorHandling({
      action: () => energyBillsApi.withdrawConsent(uploadId),
      successMessage: $_("energy.withdrawConsentSuccess"),
      errorMessage: $_("energy.withdrawConsentError"),
      onSuccess: () => loadData(),
    });
  }
</script>

{#if loading}
  <div class="p-8 text-center" data-testid="campaign-detail-loading">
    <div
      class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
    ></div>
    <p class="mt-4 text-gray-500">{$_("common.loading")}</p>
  </div>
{:else if error}
  <div class="p-4 bg-red-50 border border-red-200 rounded-md" data-testid="campaign-detail-error">
    <p class="text-sm text-red-800">❌ {error}</p>
    <button
      on:click={loadData}
      class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
    >
      {$_("common.retry")}
    </button>
  </div>
{:else if campaign}
  <div class="space-y-6" data-testid="campaign-detail">
    <!-- Header -->
    <div class="bg-white shadow-md rounded-lg p-6" data-testid="campaign-detail-header">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <h2 class="text-2xl font-bold text-gray-900 mb-2">
            {campaign.campaign_name}
          </h2>
          <CampaignStatusBadge status={campaign.status} />
        </div>
        <a
          href="/energy-campaigns"
          class="text-sm text-gray-600 hover:text-gray-800 underline"
        >
          ← {$_("common.backToList")}
        </a>
      </div>

      <p class="mt-4 text-sm text-gray-600">
        {getStatusDescription()}
      </p>

      <!-- Progress Bar -->
      <div class="mt-4">
        <div class="w-full bg-gray-200 rounded-full h-2.5">
          <div
            class="bg-indigo-600 h-2.5 rounded-full transition-all duration-300"
            style="width: {getProgressPercentage()}%"
          ></div>
        </div>
      </div>

      <!-- Campaign Info -->
      <div class="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="p-4 bg-blue-50 rounded-lg">
          <div class="text-sm text-blue-600 font-medium">{$_("energy.campaign.deadlineParticipation")}</div>
          <div class="text-lg text-blue-900">
            {formatDateShort(campaign.deadline_participation)}
          </div>
          {#if campaign.deadline_vote}
            <div class="text-xs text-blue-600 mt-1">
              {$_("energy.campaign.voteUntil")} {formatDateShort(campaign.deadline_vote)}
            </div>
          {/if}
        </div>
        <div class="p-4 bg-green-50 rounded-lg">
          <div class="text-sm text-green-600 font-medium">{$_("energy.campaign.participants")}</div>
          <div class="text-lg text-green-900">
            👥 {campaign.total_participants}
            {#if stats && !stats.k_anonymity_compliant}
              <span class="text-xs text-yellow-600">
                (min. {stats.min_participants_required} {$_("common.required")})
              </span>
            {/if}
          </div>
        </div>
        <div class="p-4 bg-purple-50 rounded-lg">
          <div class="text-sm text-purple-600 font-medium">{$_("energy.campaign.offersReceived")}</div>
          <div class="text-lg text-purple-900">
            💼 {campaign.offers_received.length}
          </div>
        </div>
      </div>
    </div>

    <!-- K-Anonymity Warning -->
    {#if stats && !stats.k_anonymity_compliant}
      <div class="p-4 bg-yellow-50 border-l-4 border-yellow-400 rounded-md">
        <div class="flex">
          <div class="flex-shrink-0">
            <span class="text-2xl">⚠️</span>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-yellow-800">
              {$_("energy.campaign.kAnonymityNotMet")}
            </h3>
            <p class="mt-1 text-sm text-yellow-700">
              {$_("energy.campaign.kAnonymityMessage", { values: { min: stats.min_participants_required, current: campaign.total_participants } })}
            </p>
            <p class="mt-1 text-xs text-yellow-600">
              {$_("energy.campaign.gdprProtection")}
            </p>
          </div>
        </div>
      </div>
    {/if}

    <!-- My Uploads Section -->
    {#if currentUnitId}
      <div class="bg-white shadow-md rounded-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium text-gray-900">
            📄 {$_("energy.myUploads")}
          </h3>
          {#if canUpload()}
            <button
              on:click={() => (showUploadForm = !showUploadForm)}
              class="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700"
            >
              {showUploadForm ? $_("common.cancel") : "➕ " + $_("energy.uploadBill")}
            </button>
          {/if}
        </div>

        {#if showUploadForm && currentUnitId}
          <EnergyBillUpload
            campaignId={campaign.id}
            unitId={currentUnitId}
            on:uploaded={handleUploadComplete}
            on:cancel={() => (showUploadForm = false)}
          />
        {:else if myUploads.length === 0}
          <p class="text-sm text-gray-500">
            {$_("energy.noUploadsYet")}
          </p>
        {:else}
          <div class="space-y-3">
            {#each myUploads as upload}
              <div
                class="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
              >
                <div>
                  <div class="text-sm font-medium text-gray-900">
                    {upload.energy_type === "Electricity"
                      ? "⚡ Électricité"
                      : upload.energy_type === "Gas"
                        ? "🔥 Gaz"
                        : "🌡️ Chauffage"}
                  </div>
                  <div class="text-xs text-gray-500">
                    {formatDateShort(upload.billing_period_start)} →
                    {formatDateShort(upload.billing_period_end)}
                  </div>
                  <div class="text-xs text-gray-500">
                    {upload.verified
                      ? "✅ " + $_("energy.verified")
                      : "⏳ " + $_("energy.verificationPending")}
                  </div>
                </div>
                <button
                  on:click={() => withdrawConsent(upload.id)}
                  data-testid="withdraw-consent-btn"
                  class="text-xs text-red-600 hover:text-red-800 underline"
                  title={$_("energy.withdrawConsentTitle")}
                >
                  🗑️ {$_("energy.withdraw")}
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Provider Offers -->
    {#if campaign.offers_received.length > 0 || campaign.status === CampaignStatus.Negotiating || campaign.status === CampaignStatus.AwaitingFinalVote}
      <ProviderOffersList
        campaignId={campaign.id}
        selectedOfferId={campaign.selected_offer_id}
        canSelect={isAdmin &&
          campaign.status === CampaignStatus.AwaitingFinalVote}
      />
    {/if}

    <!-- Statistics (Only if k-anonymity compliant) -->
    {#if stats && stats.k_anonymity_compliant}
      <div class="bg-white shadow-md rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">
          📊 {$_("energy.campaign.aggregatedStatistics")}
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="p-4 bg-indigo-50 rounded-lg">
            <div class="text-sm text-indigo-600 font-medium">
              {$_("energy.campaign.totalConsumption")}
            </div>
            <div class="text-2xl font-bold text-indigo-900">
              {stats.total_kwh_aggregated?.toLocaleString() || "N/A"} kWh
            </div>
          </div>
          <div class="p-4 bg-green-50 rounded-lg">
            <div class="text-sm text-green-600 font-medium">
              {$_("energy.campaign.averagePerUnit")}
            </div>
            <div class="text-2xl font-bold text-green-900">
              {stats.average_kwh_per_unit?.toLocaleString() || "N/A"} kWh
            </div>
          </div>
          <div class="p-4 bg-purple-50 rounded-lg">
            <div class="text-sm text-purple-600 font-medium">
              {$_("energy.campaign.estimatedSavings")}
            </div>
            <div class="text-2xl font-bold text-purple-900">
              {stats.best_offer_savings_percentage?.toFixed(1) || "N/A"}%
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}
