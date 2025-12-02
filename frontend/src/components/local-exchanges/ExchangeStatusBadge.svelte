<script lang="ts">
  import {
    ExchangeStatus,
    exchangeStatusLabels,
    exchangeStatusColors,
  } from "../../lib/api/local-exchanges";

  export let status: ExchangeStatus;

  $: config = exchangeStatusColors[status];
  $: label = exchangeStatusLabels[status];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  {label}
</span>
