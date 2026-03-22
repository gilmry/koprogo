<script lang="ts">
  import { PaymentStatus } from "../../lib/api/payments";

  export let status: PaymentStatus;

  const statusConfig: Record<
    PaymentStatus,
    { bg: string; text: string; label: string; icon: string }
  > = {
    [PaymentStatus.Pending]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Pending",
      icon: "⏳",
    },
    [PaymentStatus.Processing]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      label: "Processing",
      icon: "🔄",
    },
    [PaymentStatus.RequiresAction]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      label: "Requires Action",
      icon: "⚠️",
    },
    [PaymentStatus.Succeeded]: {
      bg: "bg-green-100",
      text: "text-green-800",
      label: "Succeeded",
      icon: "✅",
    },
    [PaymentStatus.Failed]: {
      bg: "bg-red-100",
      text: "text-red-800",
      label: "Failed",
      icon: "❌",
    },
    [PaymentStatus.Cancelled]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Cancelled",
      icon: "🚫",
    },
    [PaymentStatus.Refunded]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
      label: "Refunded",
      icon: "↩️",
    },
  };

  $: config = statusConfig[status] || statusConfig[PaymentStatus.Pending];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
