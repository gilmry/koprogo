<script lang="ts">
  import { onMount } from "svelte";
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
    try {
      loading = true;
      error = "";

      // Load campaign and stats in parallel
      const [campaignData, statsData] = await Promise.all([
        energyCampaignsApi.getById(campaignId),
        energyCampaignsApi.getStats(campaignId),
      ]);

      campaign = campaignData;
      stats = statsData;

      // Load user's uploads
      if (currentUnitId) {
        const allUploads = await energyBillsApi.getMyUploads();
        myUploads = allUploads.filter(
          (u) => u.campaign_id === campaignId && !u.deleted_at,
        );
      }
    } catch (err: any) {
      error =
        err.message || "Erreur lors du chargement des détails de la campagne";
      console.error("Failed to load campaign details:", err);
    } finally {
      loading = false;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE");
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
        return "La campagne est en cours de préparation.";
      case CampaignStatus.CollectingData:
        return "Collecte des factures d'énergie en cours. Uploadez votre facture pour participer !";
      case CampaignStatus.Negotiating:
        return "Négociation avec les fournisseurs d'énergie en cours.";
      case CampaignStatus.AwaitingFinalVote:
        return "Votez pour sélectionner l'offre finale !";
      case CampaignStatus.Finalized:
        return "L'offre a été finalisée. Signature des contrats en cours.";
      case CampaignStatus.Completed:
        return "Campagne terminée avec succès !";
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
    if (
      !confirm(
        "Êtes-vous sûr de vouloir retirer votre consentement GDPR ? Vos données seront supprimées immédiatement.",
      )
    ) {
      return;
    }

    try {
      await energyBillsApi.withdrawConsent(uploadId);
      await loadData();
      toast.success("Consentement retiré et données supprimées avec succès.");
    } catch (err: any) {
      toast.error("Erreur lors du retrait du consentement: " + err.message);
    }
  }
</script>

{#if loading}
  <div class="p-8 text-center">
    <div
      class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
    ></div>
    <p class="mt-4 text-gray-500">Chargement de la campagne...</p>
  </div>
{:else if error}
  <div class="p-4 bg-red-50 border border-red-200 rounded-md">
    <p class="text-sm text-red-800">❌ {error}</p>
    <button
      on:click={loadData}
      class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
    >
      Réessayer
    </button>
  </div>
{:else if campaign}
  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white shadow-md rounded-lg p-6">
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
          ← Retour à la liste
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
          <div class="text-sm text-blue-600 font-medium">Date limite participation</div>
          <div class="text-lg text-blue-900">
            {formatDate(campaign.deadline_participation)}
          </div>
          {#if campaign.deadline_vote}
            <div class="text-xs text-blue-600 mt-1">
              Vote jusqu'au {formatDate(campaign.deadline_vote)}
            </div>
          {/if}
        </div>
        <div class="p-4 bg-green-50 rounded-lg">
          <div class="text-sm text-green-600 font-medium">Participants</div>
          <div class="text-lg text-green-900">
            👥 {campaign.total_participants}
            {#if stats && !stats.k_anonymity_compliant}
              <span class="text-xs text-yellow-600">
                (min. {stats.min_participants_required} requis)
              </span>
            {/if}
          </div>
        </div>
        <div class="p-4 bg-purple-50 rounded-lg">
          <div class="text-sm text-purple-600 font-medium">Offres reçues</div>
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
              K-anonymité non respectée
            </h3>
            <p class="mt-1 text-sm text-yellow-700">
              Minimum <strong>{stats.min_participants_required}</strong>
              participants requis avant de publier les statistiques agrégées. Actuellement:
              <strong>{campaign.total_participants}</strong> participants.
            </p>
            <p class="mt-1 text-xs text-yellow-600">
              Protection GDPR: Les données ne seront jamais exposées tant que le
              seuil n'est pas atteint.
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
            📄 Mes factures uploadées
          </h3>
          {#if canUpload()}
            <button
              on:click={() => (showUploadForm = !showUploadForm)}
              class="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700"
            >
              {showUploadForm ? "Annuler" : "➕ Uploader une facture"}
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
            Vous n'avez pas encore uploadé de facture pour cette campagne.
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
                    {formatDate(upload.billing_period_start)} →
                    {formatDate(upload.billing_period_end)}
                  </div>
                  <div class="text-xs text-gray-500">
                    {upload.verified
                      ? "✅ Vérifié"
                      : "⏳ En attente de vérification"}
                  </div>
                </div>
                <button
                  on:click={() => withdrawConsent(upload.id)}
                  class="text-xs text-red-600 hover:text-red-800 underline"
                  title="Retirer mon consentement GDPR"
                >
                  🗑️ Retirer
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
          📊 Statistiques agrégées
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="p-4 bg-indigo-50 rounded-lg">
            <div class="text-sm text-indigo-600 font-medium">
              Consommation totale
            </div>
            <div class="text-2xl font-bold text-indigo-900">
              {stats.total_kwh_aggregated?.toLocaleString() || "N/A"} kWh
            </div>
          </div>
          <div class="p-4 bg-green-50 rounded-lg">
            <div class="text-sm text-green-600 font-medium">
              Moyenne par unité
            </div>
            <div class="text-2xl font-bold text-green-900">
              {stats.average_kwh_per_unit?.toLocaleString() || "N/A"} kWh
            </div>
          </div>
          <div class="p-4 bg-purple-50 rounded-lg">
            <div class="text-sm text-purple-600 font-medium">
              Économies estimées
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
