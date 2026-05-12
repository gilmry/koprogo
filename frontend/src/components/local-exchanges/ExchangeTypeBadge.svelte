<script lang="ts">
  // Svelte 5 runes mode
  import {
    ExchangeType,
    exchangeTypeLabels,
    exchangeTypeIcons,
  } from "../../lib/api/local-exchanges";

  let { type }: { type: ExchangeType } = $props();

  let label = $derived(exchangeTypeLabels[type]);
  let icon = $derived(exchangeTypeIcons[type]);

  const typeColors: Record<ExchangeType, { bg: string; text: string }> = {
    [ExchangeType.Service]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
    },
    [ExchangeType.ObjectLoan]: {
      bg: "bg-green-100",
      text: "text-green-800",
    },
    [ExchangeType.SharedPurchase]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
    },
  };

  let config = $derived(typeColors[type]);
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{icon}</span>
  {label}
</span>
