<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import { CampaignStatus } from "../../lib/api/energy-campaigns";

  let { status }: { status: CampaignStatus } = $props();

  function getStatusConfig() {
    const statusConfig: Record<
      CampaignStatus,
      { bg: string; text: string; label: string; icon: string }
    > = {
      [CampaignStatus.Draft]: {
        bg: "bg-gray-100",
        text: "text-gray-800",
        label: $_("energy.campaign.status.draft"),
        icon: "📝",
      },
      [CampaignStatus.AwaitingAGVote]: {
        bg: "bg-yellow-100",
        text: "text-yellow-800",
        label: "Vote AG attendu",
        icon: "⏳",
      },
      [CampaignStatus.CollectingData]: {
        bg: "bg-blue-100",
        text: "text-blue-800",
        label: $_("energy.campaign.status.collectingData"),
        icon: "📊",
      },
      [CampaignStatus.Negotiating]: {
        bg: "bg-purple-100",
        text: "text-purple-800",
        label: $_("energy.campaign.status.negotiating"),
        icon: "🤝",
      },
      [CampaignStatus.AwaitingFinalVote]: {
        bg: "bg-yellow-100",
        text: "text-yellow-800",
        label: $_("energy.campaign.status.awaitingFinalVote"),
        icon: "🗳️",
      },
      [CampaignStatus.Finalized]: {
        bg: "bg-green-100",
        text: "text-green-800",
        label: $_("energy.campaign.status.finalized"),
        icon: "✅",
      },
      [CampaignStatus.Completed]: {
        bg: "bg-emerald-100",
        text: "text-emerald-800",
        label: $_("energy.campaign.status.completed"),
        icon: "🎉",
      },
      [CampaignStatus.Cancelled]: {
        bg: "bg-red-100",
        text: "text-red-800",
        label: "Annulé",
        icon: "❌",
      },
    };
    return statusConfig[status] || statusConfig[CampaignStatus.Draft];
  }

  let config = $derived(getStatusConfig());
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
