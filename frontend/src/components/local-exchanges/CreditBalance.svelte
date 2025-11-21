<script lang="ts">
  import { onMount } from "svelte";
  import {
    localExchangesApi,
    type OwnerCreditBalance,
    CreditStatus,
    ParticipationLevel,
    participationLevelLabels,
    participationLevelColors,
    formatRating,
    getCreditStatusColor,
  } from "../../lib/api/local-exchanges";

  export let ownerId: string;
  export let buildingId: string;

  let balance: OwnerCreditBalance | null = null;
  let loading: boolean = true;
  let error: string | null = null;

  async function loadBalance() {
    try {
      loading = true;
      error = null;
      balance = await localExchangesApi.getCreditBalance(ownerId, buildingId);
    } catch (err: any) {
      error = err.message || "Impossible de charger le solde";
      console.error("Error loading credit balance:", err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadBalance();
  });

  $: participationConfig = balance
    ? participationLevelColors[balance.participation_level]
    : null;
  $: participationLabel = balance
    ? participationLevelLabels[balance.participation_level]
    : null;
  $: balanceColor = balance ? getCreditStatusColor(balance.credit_status) : "";
</script>

<div class="bg-white shadow rounded-lg p-6">
  {#if loading}
    <div class="text-center py-4">
      <div
        class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"
      ></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">❌ {error}</p>
    </div>
  {:else if balance}
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900">
          Mon Solde de Crédits
        </h3>
        <span
          class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {participationConfig?.bg} {participationConfig?.text}"
        >
          {participationLabel}
        </span>
      </div>

      <!-- Balance Display -->
      <div class="text-center py-6 bg-gradient-to-br from-blue-50 to-indigo-50 rounded-lg">
        <p class="text-sm font-medium text-gray-600 mb-1">Solde actuel</p>
        <p class="text-4xl font-bold {balanceColor}">
          {balance.balance > 0 ? "+" : ""}{balance.balance}
        </p>
        <p class="text-sm text-gray-500 mt-1">
          {balance.balance === 1 ? "heure" : "heures"}
        </p>
      </div>

      <!-- Stats Grid -->
      <div class="grid grid-cols-2 gap-4">
        <!-- Credits Earned -->
        <div class="bg-green-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-green-700 mb-1">Crédits gagnés</p>
          <p class="text-2xl font-bold text-green-900">
            {balance.credits_earned}
          </p>
          <p class="text-xs text-green-600 mt-1">Services fournis</p>
        </div>

        <!-- Credits Spent -->
        <div class="bg-orange-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-orange-700 mb-1">
            Crédits dépensés
          </p>
          <p class="text-2xl font-bold text-orange-900">
            {balance.credits_spent}
          </p>
          <p class="text-xs text-orange-600 mt-1">Services reçus</p>
        </div>

        <!-- Total Exchanges -->
        <div class="bg-blue-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-blue-700 mb-1">Échanges totaux</p>
          <p class="text-2xl font-bold text-blue-900">
            {balance.total_exchanges}
          </p>
          <p class="text-xs text-blue-600 mt-1">Transactions complétées</p>
        </div>

        <!-- Average Rating -->
        <div class="bg-yellow-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-yellow-700 mb-1">Note moyenne</p>
          <p class="text-lg font-bold text-yellow-900">
            {balance.average_rating
              ? `${balance.average_rating.toFixed(1)} ⭐`
              : "Pas encore noté"}
          </p>
          <p class="text-xs text-yellow-600 mt-1">Réputation</p>
        </div>
      </div>

      <!-- Status Explanation -->
      <div class="mt-4 p-3 bg-gray-50 rounded-md text-sm text-gray-700">
        {#if balance.credit_status === CreditStatus.Positive}
          <p>
            💚 <strong>Contributeur net</strong> - Vous avez fourni plus de
            services que vous en avez reçus. Bravo pour votre contribution à la
            communauté !
          </p>
        {:else if balance.credit_status === CreditStatus.Balanced}
          <p>
            ⚖️ <strong>Équilibre parfait</strong> - Vous avez échangé autant de
            services que vous en avez reçus.
          </p>
        {:else if balance.credit_status === CreditStatus.Negative}
          <p>
            🟠 <strong>Solde négatif</strong> - Vous avez reçu plus de services
            que vous en avez fournis. Pensez à offrir des services pour
            rééquilibrer.
          </p>
        {/if}
      </div>

      <!-- Legal Notice (Belgian SEL Context) -->
      <div class="mt-4 p-3 bg-blue-50 border-l-4 border-blue-400 text-xs text-blue-800">
        <p>
          <strong>ℹ️ Contexte légal belge:</strong> Les SEL sont légaux en
          Belgique et non taxables si non-commerciaux (troc). Monnaie temps: 1
          heure = 1 crédit. Solde négatif autorisé (modèle de confiance
          communautaire).
        </p>
      </div>
    </div>
  {/if}
</div>
