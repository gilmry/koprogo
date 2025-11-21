<script lang="ts">
  import { PollStatus } from "../../lib/api/polls";

  export let status: PollStatus;

  const statusConfig: Record<
    PollStatus,
    { bg: string; text: string; label: string; icon: string }
  > = {
    [PollStatus.Draft]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      label: "Brouillon",
      icon: "📝",
    },
    [PollStatus.Active]: {
      bg: "bg-green-100",
      text: "text-green-800",
      label: "En cours",
      icon: "🗳️",
    },
    [PollStatus.Closed]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      label: "Clôturé",
      icon: "✅",
    },
    [PollStatus.Cancelled]: {
      bg: "bg-red-100",
      text: "text-red-800",
      label: "Annulé",
      icon: "❌",
    },
  };

  $: config = statusConfig[status] || statusConfig[PollStatus.Draft];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
