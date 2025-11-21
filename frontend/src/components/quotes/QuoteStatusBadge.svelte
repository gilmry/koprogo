<script lang="ts">
  import { QuoteStatus } from "../../lib/api/quotes";

  export let status: QuoteStatus;

  const statusConfig: Record<
    QuoteStatus,
    { bg: string; text: string; label: string; icon: string }
  > = {
    [QuoteStatus.Requested]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      label: "Requested",
      icon: "📋",
    },
    [QuoteStatus.Received]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
      label: "Received",
      icon: "📨",
    },
    [QuoteStatus.UnderReview]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      label: "Under Review",
      icon: "🔍",
    },
    [QuoteStatus.Accepted]: {
      bg: "bg-green-100",
      text: "text-green-800",
      label: "Accepted",
      icon: "✅",
    },
    [QuoteStatus.Rejected]: {
      bg: "bg-red-100",
      text: "text-red-800",
      label: "Rejected",
      icon: "❌",
    },
    [QuoteStatus.Expired]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Expired",
      icon: "⏰",
    },
    [QuoteStatus.Withdrawn]: {
      bg: "bg-orange-100",
      text: "text-orange-800",
      label: "Withdrawn",
      icon: "↩️",
    },
  };

  $: config = statusConfig[status] || statusConfig[QuoteStatus.Requested];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
