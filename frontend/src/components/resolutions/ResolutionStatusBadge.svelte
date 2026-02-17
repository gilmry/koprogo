<script lang="ts">
  import { ResolutionStatus } from '../../lib/api/resolutions';

  export let status: ResolutionStatus;

  const config: Record<ResolutionStatus, { bg: string; text: string; label: string; icon: string }> = {
    [ResolutionStatus.Pending]: { bg: 'bg-yellow-100', text: 'text-yellow-800', label: 'En attente', icon: '⏳' },
    [ResolutionStatus.Adopted]: { bg: 'bg-green-100', text: 'text-green-800', label: 'Adoptée', icon: '✅' },
    [ResolutionStatus.Rejected]: { bg: 'bg-red-100', text: 'text-red-800', label: 'Rejetée', icon: '❌' },
  };

  $: badge = config[status] || config[ResolutionStatus.Pending];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {badge.bg} {badge.text}">
  <span class="mr-1">{badge.icon}</span>
  {badge.label}
</span>
