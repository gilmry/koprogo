<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { PollStatus } from "../../lib/api/polls";

  export let status: PollStatus;

  const statusConfig: Record<
    PollStatus,
    { bg: string; text: string; labelKey: string; icon: string }
  > = {
    [PollStatus.Draft]: {
      bg: "bg-gray-100",
      text: "text-gray-800",
      labelKey: "polls.status.draft",
      icon: "📝",
    },
    [PollStatus.Active]: {
      bg: "bg-green-100",
      text: "text-green-800",
      labelKey: "polls.status.active",
      icon: "🗳️",
    },
    [PollStatus.Closed]: {
      bg: "bg-blue-100",
      text: "text-blue-800",
      labelKey: "polls.status.closed",
      icon: "✅",
    },
    [PollStatus.Cancelled]: {
      bg: "bg-red-100",
      text: "text-red-800",
      labelKey: "polls.status.cancelled",
      icon: "❌",
    },
  };

  $: config = statusConfig[status] || statusConfig[PollStatus.Draft];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {$_(config.labelKey)}
</span>
