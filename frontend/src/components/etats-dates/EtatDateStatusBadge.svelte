<script lang="ts">
  import { _ } from '../../lib/i18n';

  export let status: string;

  function getBadge(s: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'requested': { class: 'bg-blue-100 text-blue-800', label: $_('etatsDate.status.requested') },
      'in_progress': { class: 'bg-yellow-100 text-yellow-800', label: $_('etatsDate.status.inProgress') },
      'generated': { class: 'bg-green-100 text-green-800', label: $_('etatsDate.status.generated') },
      'delivered': { class: 'bg-purple-100 text-purple-800', label: $_('etatsDate.status.delivered') },
      'expired': { class: 'bg-red-100 text-red-800', label: $_('etatsDate.status.expired') },
    };
    return badges[s] || { class: 'bg-gray-100 text-gray-800', label: s };
  }

  $: badge = getBadge(status);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.class}">
  {badge.label}
</span>
