<script lang="ts">
  // Svelte 5 runes mode
  import {
    ExchangeStatus,
    exchangeStatusLabels,
    exchangeStatusColors,
  } from "../../lib/api/local-exchanges";

  let { status }: { status: ExchangeStatus } = $props();

  let config = $derived(exchangeStatusColors[status]);
  let label = $derived(exchangeStatusLabels[status]);
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  {label}
</span>
