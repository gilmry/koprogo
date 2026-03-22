<script lang="ts">
  import { _ } from "svelte-i18n";
  import { QuoteStatus } from "../../lib/api/quotes";

  export let status: QuoteStatus;

  function getStatusLabel(status: QuoteStatus): string {
    switch (status) {
      case QuoteStatus.Requested: return $_("quotes.status.requested");
      case QuoteStatus.Received: return $_("quotes.status.received");
      case QuoteStatus.UnderReview: return $_("quotes.status.underReview");
      case QuoteStatus.Accepted: return $_("quotes.status.accepted");
      case QuoteStatus.Rejected: return $_("quotes.status.rejected");
      case QuoteStatus.Expired: return $_("quotes.status.expired");
      case QuoteStatus.Withdrawn: return $_("quotes.status.withdrawn");
      default: return status;
    }
  }

  const statusConfig: Record<
    QuoteStatus,
    { bg: string; text: string; icon: string }
  > = {
    [QuoteStatus.Requested]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      icon: "📋",
    },
    [QuoteStatus.Received]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
      icon: "📨",
    },
    [QuoteStatus.UnderReview]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      icon: "🔍",
    },
    [QuoteStatus.Accepted]: {
      bg: "bg-green-100",
      text: "text-green-800",
      icon: "✅",
    },
    [QuoteStatus.Rejected]: {
      bg: "bg-red-100",
      text: "text-red-800",
      icon: "❌",
    },
    [QuoteStatus.Expired]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      icon: "⏰",
    },
    [QuoteStatus.Withdrawn]: {
      bg: "bg-orange-100",
      text: "text-orange-800",
      icon: "↩️",
    },
  };

  $: config = statusConfig[status] || statusConfig[QuoteStatus.Requested];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {getStatusLabel(status)}
</span>
