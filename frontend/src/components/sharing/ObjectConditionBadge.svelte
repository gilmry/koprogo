<script lang="ts">
  // Svelte 5 runes mode
  import { ObjectCondition } from "../../lib/api/sharing";

  let { condition }: { condition: ObjectCondition } = $props();

  const conditionConfig: Record<
    ObjectCondition,
    { label: string; class: string }
  > = {
    [ObjectCondition.Excellent]: {
      label: "Excellent",
      class: "bg-green-100 text-green-800",
    },
    [ObjectCondition.Good]: {
      label: "Good",
      class: "bg-blue-100 text-blue-800",
    },
    [ObjectCondition.Fair]: {
      label: "Fair",
      class: "bg-yellow-100 text-yellow-800",
    },
    [ObjectCondition.Used]: {
      label: "Used",
      class: "bg-orange-100 text-orange-800",
    },
  };

  let config = $derived(conditionConfig[condition]);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {config.label}
</span>
