<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { quotesApi, type QuoteComparison } from "../../lib/api/quotes";
  import QuoteStatusBadge from "./QuoteStatusBadge.svelte";
  import { withLoadingState } from '../../lib/utils/error.utils';
  import { formatDateShort } from '../../lib/utils/date.utils';
  import { formatAmount } from '../../lib/utils/finance.utils';

  export let quoteIds: string[];

  let comparison: QuoteComparison | null = null;
  let loading = true;
  let error = '';

  onMount(async () => {
    await loadComparison();
  });

  async function loadComparison() {
    await withLoadingState({
      action: () => quotesApi.compare(quoteIds),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      errorMessage: $_("quotes.comparison.loadError"),
      onSuccess: (data) => { comparison = data; },
    });
  }

  function formatComparisonAmount(amountCents: number | undefined): string {
    if (!amountCents) return "N/A";
    return formatAmount(amountCents);
  }

  function getScoreClass(score: number): string {
    if (score >= 80) return "text-green-600 font-semibold";
    if (score >= 60) return "text-yellow-600";
    return "text-red-600";
  }
</script>

<div class="bg-white shadow rounded-lg">
  <div class="px-6 py-4 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      {$_("quotes.comparison.title")}
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      {$_("quotes.comparison.scoring")}
    </p>
  </div>

  <div class="p-6">
    {#if loading}
      <div class="text-center py-12 text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">{$_("quotes.comparison.loading")}</p>
      </div>
    {:else if !comparison}
      <div class="text-center py-12 text-red-600">
        {$_("quotes.comparison.loadError")}
      </div>
    {:else}
      <!-- Belgian Law Compliance -->
      <div class="mb-6 p-4 rounded-lg {comparison.complies_with_belgian_law ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}">
        <div class="flex items-start">
          <div class="flex-shrink-0">
            {#if comparison.complies_with_belgian_law}
              <svg
                class="h-5 w-5 text-green-400"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clip-rule="evenodd"
                />
              </svg>
            {:else}
              <svg
                class="h-5 w-5 text-red-400"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clip-rule="evenodd"
                />
              </svg>
            {/if}
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium {comparison.complies_with_belgian_law ? 'text-green-800' : 'text-red-800'}">
              {#if comparison.complies_with_belgian_law}
                ✅ {$_("quotes.comparison.compliant")}
              {:else}
                ⚠️ {$_("quotes.comparison.recommendation")}
              {/if}
            </h3>
            <p class="mt-1 text-sm {comparison.complies_with_belgian_law ? 'text-green-700' : 'text-red-700'}">
              {comparison.recommendation}
            </p>
          </div>
        </div>
      </div>

      <!-- Comparison Table -->
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200" data-testid="comparison-table">
          <thead class="bg-gray-50">
            <tr>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("quotes.comparison.contractor")}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("quotes.comparison.price")}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("quotes.comparison.duration")}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("quotes.comparison.warranty")}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("common.status")}
              </th>
              <th
                class="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_("quotes.comparison.totalScore")}
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each comparison.quotes as item, index (item.quote.id)}
              <tr class={index === 0 ? "bg-green-50" : ""} data-testid="comparison-row">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="flex items-center">
                    {#if index === 0}
                      <span class="text-xl mr-2">🏆</span>
                    {/if}
                    <div>
                      <div class="text-sm font-medium text-gray-900">
                        {item.quote.contractor_name || "Contractor"}
                      </div>
                      <div class="text-xs text-gray-500">
                        Rating: {item.quote.contractor_rating || "N/A"}/100
                      </div>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm font-medium text-gray-900">
                    {formatComparisonAmount(item.quote.amount_incl_vat_cents)}
                  </div>
                  <div class="text-xs text-gray-500">
                    Score: {item.price_score.toFixed(1)}/40
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900">
                    {item.quote.estimated_duration_days || "N/A"} days
                  </div>
                  <div class="text-xs text-gray-500">
                    Score: {item.delay_score.toFixed(1)}/30
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900">
                    {item.quote.warranty_years || "N/A"} years
                  </div>
                  <div class="text-xs text-gray-500">
                    Score: {item.warranty_score.toFixed(1)}/20
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <QuoteStatusBadge status={item.quote.status} />
                  {#if item.quote.validity_date}
                    <div class="text-xs text-gray-500 mt-1">
                      {$_("quotes.comparison.validUntil")}: {formatDateShort(item.quote.validity_date)}
                    </div>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-center" data-testid="comparison-score">
                  <div class="text-2xl font-bold {getScoreClass(item.score)}">
                    {item.score.toFixed(1)}
                  </div>
                  <div class="text-xs text-gray-500 mt-1">
                    Reputation: {item.reputation_score.toFixed(1)}/10
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Legend -->
      <div class="mt-6 pt-6 border-t border-gray-200">
        <h3 class="text-sm font-medium text-gray-900 mb-2">{$_("quotes.comparison.methodology")}</h3>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <span class="font-medium text-blue-600">{$_("quotes.comparison.priceWeight")}</span>
            <p class="text-gray-600 text-xs mt-1">
              {$_("quotes.comparison.priceDesc")}
            </p>
          </div>
          <div>
            <span class="font-medium text-yellow-600">{$_("quotes.comparison.delayWeight")}</span>
            <p class="text-gray-600 text-xs mt-1">
              {$_("quotes.comparison.delayDesc")}
            </p>
          </div>
          <div>
            <span class="font-medium text-green-600">{$_("quotes.comparison.warrantyWeight")}</span>
            <p class="text-gray-600 text-xs mt-1">
              {$_("quotes.comparison.warrantyDesc")}
            </p>
          </div>
          <div>
            <span class="font-medium text-purple-600">{$_("quotes.comparison.reputationWeight")}</span>
            <p class="text-gray-600 text-xs mt-1">{$_("quotes.comparison.reputationDesc")}</p>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
