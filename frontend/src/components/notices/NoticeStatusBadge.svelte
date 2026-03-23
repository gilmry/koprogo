<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { NoticeStatus } from "../../lib/api/notices";

  export let status: NoticeStatus;

  const statusConfig: Record<NoticeStatus, { labelKey: string; class: string }> = {
    [NoticeStatus.Draft]: {
      labelKey: "notices.status_draft",
      class: "bg-yellow-100 text-yellow-800",
    },
    [NoticeStatus.Published]: {
      labelKey: "notices.status_published",
      class: "bg-green-100 text-green-800",
    },
    [NoticeStatus.Archived]: {
      labelKey: "notices.status_archived",
      class: "bg-gray-100 text-gray-600",
    },
    [NoticeStatus.Expired]: {
      labelKey: "notices.status_expired",
      class: "bg-gray-100 text-gray-800",
    },
  };

  $: config = statusConfig[status];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {$_("notices." + status.toLowerCase())}
</span>
