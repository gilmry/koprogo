<script lang="ts">
  export let status: string;

  function getBadge(s: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'draft': { class: 'bg-gray-100 text-gray-800', label: 'Brouillon' },
      'submitted': { class: 'bg-blue-100 text-blue-800', label: 'Soumis' },
      'approved': { class: 'bg-green-100 text-green-800', label: 'Approuve' },
      'rejected': { class: 'bg-red-100 text-red-800', label: 'Rejete' },
      'archived': { class: 'bg-yellow-100 text-yellow-800', label: 'Archive' },
    };
    return badges[s] || { class: 'bg-gray-100 text-gray-800', label: s };
  }

  $: badge = getBadge(status);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.class}">
  {badge.label}
</span>
