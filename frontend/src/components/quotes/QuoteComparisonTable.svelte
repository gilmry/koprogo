<script lang="ts">
  import { onMount } from "svelte";
  import { quotesApi, type QuoteComparison } from "../../lib/api/quotes";
  import { toast } from "../../stores/toast";
  import QuoteStatusBadge from "./QuoteStatusBadge.svelte";

  export let quoteIds: string[];

  let comparison: QuoteComparison | null = null;
  let loading = true;

  onMount(async () => {
    await loadComparison();
  });

  async function loadComparison() {
    try {
      loading = true;
      comparison = await quotesApi.compare(quoteIds);
    } catch (err: any) {
      toast.error(err.message || "Failed to load quote comparison");
    } finally {
      loading = false;
    }
  }

  function formatAmount(amountCents: number | undefined): string {
    if (!amountCents) return "N/A";
    const amount = amountCents / 100;
    return new Intl.NumberFormat("nl-BE", {
      style: "currency",
      currency: "EUR",
    }).format(amount);
  }

  function formatDate(dateString: string | undefined): string {
    if (!dateString) return "N/A";
    return new Date(dateString).toLocaleDateString("nl-BE", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
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
      Quote Comparison - Belgian 3-Quote Algorithm
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      Automatic scoring: Price 40% • Delay 30% • Warranty 20% • Reputation 10%
    </p>
  </div>

  <div class="p-6">
    {#if loading}
      <div class="text-center py-12 text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">Analyzing quotes...</p>
      </div>
    {:else if !comparison}
      <div class="text-center py-12 text-red-600">
        Failed to load comparison
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
                ✅ Complies with Belgian Law
              {:else}
                ⚠️ Belgian Law Requirement: 3 quotes needed for works >5000€
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
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Contractor
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Price (Incl. VAT)
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Duration
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Warranty
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
              <th
                class="px-6 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Total Score
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each comparison.quotes as item, index (item.quote.id)}
              <tr class={index === 0 ? "bg-green-50" : ""}>
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
                    {formatAmount(item.quote.amount_incl_vat_cents)}
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
                      Valid until: {formatDate(item.quote.validity_date)}
                    </div>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-center">
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
        <h3 class="text-sm font-medium text-gray-900 mb-2">Scoring Methodology</h3>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
          <div>
            <span class="font-medium text-blue-600">Price (40%)</span>
            <p class="text-gray-600 text-xs mt-1">
              Lower price = higher score (inverted)
            </p>
          </div>
          <div>
            <span class="font-medium text-yellow-600">Delay (30%)</span>
            <p class="text-gray-600 text-xs mt-1">
              Shorter duration = higher score
            </p>
          </div>
          <div>
            <span class="font-medium text-green-600">Warranty (20%)</span>
            <p class="text-gray-600 text-xs mt-1">
              Longer warranty = higher score
            </p>
          </div>
          <div>
            <span class="font-medium text-purple-600">Reputation (10%)</span>
            <p class="text-gray-600 text-xs mt-1">Contractor rating 0-100</p>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
