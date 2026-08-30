<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "svelte-i18n";
  import { NoticeType } from "../../lib/api/notices";

  let { type }: { type: NoticeType } = $props();

  const typeConfig: Record<
    NoticeType,
    { class: string; icon: string }
  > = {
    [NoticeType.Announcement]: {
      class: "bg-blue-100 text-blue-800",
      icon: "📢",
    },
    [NoticeType.Event]: {
      class: "bg-pink-100 text-pink-800",
      icon: "🎉",
    },
    [NoticeType.LostAndFound]: {
      class: "bg-yellow-100 text-yellow-800",
      icon: "🔎",
    },
    [NoticeType.ClassifiedAd]: {
      class: "bg-green-100 text-green-800",
      icon: "💰",
    },
  };

  let config = $derived(typeConfig[type]);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {$_(`notices.noticeType.${type}`)}
</span>
