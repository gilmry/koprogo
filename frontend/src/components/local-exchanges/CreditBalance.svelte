<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
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
  import { withLoadingState } from "../../lib/utils/error.utils";

  export let ownerId: string;
  export let buildingId: string;

  let balance: OwnerCreditBalance | null = null;
  let loading: boolean = true;
  let error: string | null = null;

  async function loadBalance() {
    await withLoadingState({
      action: () => localExchangesApi.getCreditBalance(ownerId, buildingId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => balance = data,
      errorMessage: $_("exchanges.loadBalanceError"),
    });
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

<div class="bg-white shadow rounded-lg p-6" data-testid="credit-balance">
  {#if loading}
    <div class="text-center py-4" data-testid="credit-balance-loading">
      <div
        class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"
      ></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4" data-testid="credit-balance-error">
      <p class="text-red-800">❌ {error}</p>
    </div>
  {:else if balance}
    <div class="space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-900">
          {$_("exchanges.myBalance")}
        </h3>
        <span
          class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {participationConfig?.bg} {participationConfig?.text}"
        >
          {participationLabel}
        </span>
      </div>

      <!-- Balance Display -->
      <div class="text-center py-6 bg-gradient-to-br from-blue-50 to-indigo-50 rounded-lg" data-testid="credit-balance-display">
        <p class="text-sm font-medium text-gray-600 mb-1">{$_("exchanges.currentBalance")}</p>
        <p class="text-4xl font-bold {balanceColor}">
          {balance.balance > 0 ? "+" : ""}{balance.balance}
        </p>
        <p class="text-sm text-gray-500 mt-1">
          {balance.balance === 1 ? $_("exchanges.hour") : $_("exchanges.hours")}
        </p>
      </div>

      <!-- Stats Grid -->
      <div class="grid grid-cols-2 gap-4">
        <!-- Credits Earned -->
        <div class="bg-green-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-green-700 mb-1">{$_("exchanges.creditsEarned")}</p>
          <p class="text-2xl font-bold text-green-900">
            {balance.credits_earned}
          </p>
          <p class="text-xs text-green-600 mt-1">{$_("exchanges.servicesProvided")}</p>
        </div>

        <!-- Credits Spent -->
        <div class="bg-orange-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-orange-700 mb-1">
            {$_("exchanges.creditsSpent")}
          </p>
          <p class="text-2xl font-bold text-orange-900">
            {balance.credits_spent}
          </p>
          <p class="text-xs text-orange-600 mt-1">{$_("exchanges.servicesReceived")}</p>
        </div>

        <!-- Total Exchanges -->
        <div class="bg-blue-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-blue-700 mb-1">{$_("exchanges.totalExchanges")}</p>
          <p class="text-2xl font-bold text-blue-900">
            {balance.total_exchanges}
          </p>
          <p class="text-xs text-blue-600 mt-1">{$_("exchanges.completedTransactions")}</p>
        </div>

        <!-- Average Rating -->
        <div class="bg-yellow-50 p-4 rounded-lg">
          <p class="text-xs font-medium text-yellow-700 mb-1">{$_("exchanges.averageRating")}</p>
          <p class="text-lg font-bold text-yellow-900">
            {balance.average_rating
              ? `${balance.average_rating.toFixed(1)} ⭐`
              : $_("exchanges.notRatedYet")}
          </p>
          <p class="text-xs text-yellow-600 mt-1">{$_("exchanges.reputation")}</p>
        </div>
      </div>

      <!-- Status Explanation -->
      <div class="mt-4 p-3 bg-gray-50 rounded-md text-sm text-gray-700">
        {#if balance.credit_status === CreditStatus.Positive}
          <p>
            {$_("exchanges.statusPositive")}
          </p>
        {:else if balance.credit_status === CreditStatus.Balanced}
          <p>
            {$_("exchanges.statusBalanced")}
          </p>
        {:else if balance.credit_status === CreditStatus.Negative}
          <p>
            {$_("exchanges.statusNegative")}
          </p>
        {/if}
      </div>

      <!-- Legal Notice (Belgian SEL Context) -->
      <div class="mt-4 p-3 bg-blue-50 border-l-4 border-blue-400 text-xs text-blue-800">
        <p>
          {$_("exchanges.legalNoticeBelgian")}
        </p>
      </div>
    </div>
  {/if}
</div>
