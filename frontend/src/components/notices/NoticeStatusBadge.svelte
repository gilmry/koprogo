<script lang="ts">
  import { NoticeStatus } from "../../lib/api/notices";

  export let status: NoticeStatus;

  const statusConfig: Record<NoticeStatus, { label: string; class: string }> = {
    [NoticeStatus.Draft]: {
      label: "Draft",
      class: "bg-yellow-100 text-yellow-800",
    },
    [NoticeStatus.Published]: {
      label: "Published",
      class: "bg-green-100 text-green-800",
    },
    [NoticeStatus.Archived]: {
      label: "Archived",
      class: "bg-gray-100 text-gray-600",
    },
    [NoticeStatus.Expired]: {
      label: "Expired",
      class: "bg-gray-100 text-gray-800",
    },
  };

  $: config = statusConfig[status];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {config.label}
</span>
