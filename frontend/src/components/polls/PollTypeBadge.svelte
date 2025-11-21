<script lang="ts">
  import { PollType } from "../../lib/api/polls";

  export let type: PollType;

  const typeConfig: Record<
    PollType,
    { bg: string; text: string; label: string; icon: string }
  > = {
    [PollType.YesNo]: {
      bg: "bg-purple-100",
      text: "text-purple-800",
      label: "Oui/Non",
      icon: "👍👎",
    },
    [PollType.MultipleChoice]: {
      bg: "bg-indigo-100",
      text: "text-indigo-800",
      label: "Choix multiple",
      icon: "☑️",
    },
    [PollType.Rating]: {
      bg: "bg-yellow-100",
      text: "text-yellow-800",
      label: "Notation",
      icon: "⭐",
    },
    [PollType.OpenEnded]: {
      bg: "bg-pink-100",
      text: "text-pink-800",
      label: "Texte libre",
      icon: "💬",
    },
  };

  $: config = typeConfig[type] || typeConfig[PollType.YesNo];
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
