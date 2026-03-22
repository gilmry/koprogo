<script lang="ts">
  import { _ } from "svelte-i18n";
  import { CampaignStatus } from "../../lib/api/energy-campaigns";

  export let status: CampaignStatus;

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
    };
    return statusConfig[status] || statusConfig[CampaignStatus.Draft];
  }

  $: config = getStatusConfig();
</script>

<span
  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.bg} {config.text}"
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
