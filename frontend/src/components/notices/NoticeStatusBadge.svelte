<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import { NoticeStatus } from "../../lib/api/notices";

  let { status }: { status: NoticeStatus } = $props();

  const statusConfig: Record<
    NoticeStatus,
    { labelKey: string; class: string }
  > = {
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

  let config = $derived(statusConfig[status]);
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}"
>
  <!-- La clé vient de `statusConfig`, pas d'une interpolation.
       `"notices." + status.toLowerCase()` donnait `notices.draft`, une clé qui
       n'existe pas : c'est la CLÉ BRUTE qui s'affichait sur chaque carte
       d'annonce, alors que `labelKey` juste au-dessus porte la bonne
       (`notices.status_draft`, présente dans les quatre langues). Constaté en
       recette le 2026-09-06, RN-23, issue #786. -->
  {$_(config.labelKey)}
</span>
