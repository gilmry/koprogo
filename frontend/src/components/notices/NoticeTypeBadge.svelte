<script lang="ts">
  import { NoticeType } from "../../lib/api/notices";

  export let type: NoticeType;

  const typeConfig: Record<
    NoticeType,
    { label: string; class: string; icon: string }
  > = {
    [NoticeType.Announcement]: {
      label: "Announcement",
      class: "bg-blue-100 text-blue-800",
      icon: "📢",
    },
    [NoticeType.Event]: {
      label: "Event",
      class: "bg-pink-100 text-pink-800",
      icon: "🎉",
    },
    [NoticeType.LostAndFound]: {
      label: "Lost & Found",
      class: "bg-yellow-100 text-yellow-800",
      icon: "🔎",
    },
    [NoticeType.ClassifiedAd]: {
      label: "Classified Ad",
      class: "bg-green-100 text-green-800",
      icon: "💰",
    },
  };

  $: config = typeConfig[type];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
