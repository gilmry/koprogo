<script lang="ts">
  import { TicketStatus } from "../../lib/api/tickets";

  export let status: TicketStatus;

  const statusColors: Record<
    TicketStatus,
    { bg: string; text: string; label: string }
  > = {
    [TicketStatus.Open]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      label: "Open",
    },
    [TicketStatus.Assigned]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
      label: "Assigned",
    },
    [TicketStatus.InProgress]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      label: "In Progress",
    },
    [TicketStatus.Resolved]: {
      bg: "bg-green-100",
      text: "text-green-800",
      label: "Resolved",
    },
    [TicketStatus.Closed]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Closed",
    },
    [TicketStatus.Cancelled]: {
      bg: "bg-red-100",
      text: "text-red-800",
      label: "Cancelled",
    },
  };

  $: config = statusColors[status] || statusColors[TicketStatus.Open];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  {config.label}
</span>
