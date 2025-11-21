<script lang="ts">
  import { ObjectCondition } from "../../lib/api/sharing";

  export let condition: ObjectCondition;

  const conditionConfig: Record<
    ObjectCondition,
    { label: string; class: string }
  > = {
    [ObjectCondition.New]: {
      label: "New",
      class: "bg-green-100 text-green-800",
    },
    [ObjectCondition.LikeNew]: {
      label: "Like New",
      class: "bg-blue-100 text-blue-800",
    },
    [ObjectCondition.Good]: {
      label: "Good",
      class: "bg-yellow-100 text-yellow-800",
    },
    [ObjectCondition.Fair]: {
      label: "Fair",
      class: "bg-orange-100 text-orange-800",
    },
    [ObjectCondition.Poor]: {
      label: "Poor",
      class: "bg-red-100 text-red-800",
    },
  };

  $: config = conditionConfig[condition];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {config.label}
</span>
