<script lang="ts">
  import { TicketPriority } from "../../lib/api/tickets";

  export let priority: TicketPriority;

  const priorityColors: Record<
    TicketPriority,
    { bg: string; text: string; label: string; icon: string }
  > = {
    [TicketPriority.Critical]: {
      bg: "bg-red-100",
      text: "text-red-800",
      label: "Critical (1h)",
      icon: "🔴",
    },
    [TicketPriority.Urgent]: {
      bg: "bg-orange-100",
      text: "text-orange-800",
      label: "Urgent (4h)",
      icon: "🟠",
    },
    [TicketPriority.High]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      label: "High (24h)",
      icon: "🟡",
    },
    [TicketPriority.Medium]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      label: "Medium (3d)",
      icon: "🔵",
    },
    [TicketPriority.Low]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Low (7d)",
      icon: "⚪",
    },
  };

  $: config = priorityColors[priority] || priorityColors[TicketPriority.Medium];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
