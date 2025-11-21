<script lang="ts">
  import { AvailabilityStatus } from "../../lib/api/sharing";

  export let status: AvailabilityStatus;

  const statusConfig: Record<
    AvailabilityStatus,
    { label: string; class: string; icon: string }
  > = {
    [AvailabilityStatus.Available]: {
      label: "Available",
      class: "bg-green-100 text-green-800",
      icon: "✅",
    },
    [AvailabilityStatus.OnLoan]: {
      label: "On Loan",
      class: "bg-blue-100 text-blue-800",
      icon: "📤",
    },
    [AvailabilityStatus.Reserved]: {
      label: "Reserved",
      class: "bg-yellow-100 text-yellow-800",
      icon: "🔒",
    },
    [AvailabilityStatus.Unavailable]: {
      label: "Unavailable",
      class: "bg-red-100 text-red-800",
      icon: "❌",
    },
    [AvailabilityStatus.Retired]: {
      label: "Retired",
      class: "bg-gray-100 text-gray-800",
      icon: "🗄️",
    },
  };

  $: config = statusConfig[status];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
