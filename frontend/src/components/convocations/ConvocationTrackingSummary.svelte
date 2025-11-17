<script lang="ts">
  import { onMount } from "svelte";
  import { convocationsApi, type TrackingSummary } from "../../lib/api/convocations";
  import { addToast } from "../../stores/toast";

  export let convocationId: string;

  let summary: TrackingSummary | null = null;
  let loading = true;

  onMount(async () => {
    await loadSummary();
  });

  async function loadSummary() {
    try {
      loading = true;
      summary = await convocationsApi.getTrackingSummary(convocationId);
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to load tracking summary",
        type: "error",
      });
    } finally {
      loading = false;
    }
  }

  function formatPercentage(rate: number): string {
    return `${(rate * 100).toFixed(1)}%`;
  }
</script>

<div class="bg-white shadow rounded-lg p-6">
  <h3 class="text-lg font-semibold text-gray-900 mb-4">Tracking Summary</h3>

  {#if loading}
    <div class="text-center py-8 text-gray-500">Loading...</div>
  {:else if summary}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Recipients -->
      <div class="bg-gray-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-gray-500 mb-1">Total Recipients</dt>
        <dd class="text-2xl font-bold text-gray-900">{summary.total_recipients}</dd>
      </div>

      <!-- Email Opened -->
      <div class="bg-blue-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-blue-700 mb-1">📧 Opened</dt>
        <dd class="text-2xl font-bold text-blue-900">{summary.email_opened}</dd>
        <dd class="text-xs text-blue-600 mt-1">
          {formatPercentage(summary.opening_rate)} opening rate
        </dd>
      </div>

      <!-- Will Attend -->
      <div class="bg-green-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-green-700 mb-1">✅ Will Attend</dt>
        <dd class="text-2xl font-bold text-green-900">{summary.will_attend}</dd>
        <dd class="text-xs text-green-600 mt-1">
          {formatPercentage(summary.attendance_rate)} confirmed
        </dd>
      </div>

      <!-- Failed Emails -->
      {#if summary.email_failed > 0}
        <div class="bg-red-50 rounded-lg p-4">
          <dt class="text-sm font-medium text-red-700 mb-1">❌ Failed</dt>
          <dd class="text-2xl font-bold text-red-900">{summary.email_failed}</dd>
        </div>
      {/if}
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">No tracking data available</div>
  {/if}
</div>
