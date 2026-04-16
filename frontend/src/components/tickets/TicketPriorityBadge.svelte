<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-602)
  import { TicketPriority } from "../../lib/api/tickets";
  import { _ } from "../../lib/i18n";

  let { priority }: { priority: TicketPriority } = $props();

  const priorityColors: Record<TicketPriority, { bg: string; text: string; key: string; icon: string }> = {
    [TicketPriority.Critical]: { bg: "bg-red-100", text: "text-red-800", key: "tickets.priorities.critical", icon: "🔴" },
    [TicketPriority.High]: { bg: "bg-yellow-100", text: "text-yellow-800", key: "tickets.priorities.high", icon: "🟡" },
    [TicketPriority.Medium]: { bg: "bg-blue-100", text: "text-blue-800", key: "tickets.priorities.medium", icon: "🔵" },
    [TicketPriority.Low]: { bg: "bg-gray-100", text: "text-gray-800", key: "tickets.priorities.low", icon: "⚪" },
  };

  let config = $derived(priorityColors[priority] || priorityColors[TicketPriority.Medium]);
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {$_(config.key)}
</span>
