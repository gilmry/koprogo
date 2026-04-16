<script lang="ts">
  import { TicketStatus } from "../../lib/api/tickets";
  import { _ } from "../../lib/i18n";

  export let status: TicketStatus;

  const statusColors: Record<TicketStatus, { bg: string; text: string; key: string }> = {
    [TicketStatus.Open]: { bg: "bg-blue-100", text: "text-blue-800", key: "tickets.statuses.open" },
    [TicketStatus.InProgress]: { bg: "bg-yellow-100", text: "text-yellow-800", key: "tickets.statuses.inProgress" },
    [TicketStatus.Resolved]: { bg: "bg-green-100", text: "text-green-800", key: "tickets.statuses.resolved" },
    [TicketStatus.Closed]: { bg: "bg-gray-100", text: "text-gray-800", key: "tickets.statuses.closed" },
    [TicketStatus.Cancelled]: { bg: "bg-red-100", text: "text-red-800", key: "tickets.statuses.cancelled" },
  };

  $: config = statusColors[status] || statusColors[TicketStatus.Open];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  {$_(config.key)}
</span>
