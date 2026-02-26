<script lang="ts">
  import { ExpertiseLevel } from "../../lib/api/skills";

  export let level: ExpertiseLevel;

  const levelConfig: Record<
    ExpertiseLevel,
    { label: string; class: string; stars: string }
  > = {
    [ExpertiseLevel.Beginner]: {
      label: "Beginner",
      class: "bg-gray-100 text-gray-700",
      stars: "⭐",
    },
    [ExpertiseLevel.Intermediate]: {
      label: "Intermediate",
      class: "bg-blue-100 text-blue-700",
      stars: "⭐⭐",
    },
    [ExpertiseLevel.Advanced]: {
      label: "Advanced",
      class: "bg-green-100 text-green-700",
      stars: "⭐⭐⭐",
    },
    [ExpertiseLevel.Expert]: {
      label: "Expert",
      class: "bg-purple-100 text-purple-700",
      stars: "⭐⭐⭐⭐",
    },
  };

  $: config = levelConfig[level] || levelConfig[ExpertiseLevel.Beginner];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.stars}</span>
  {config.label}
</span>
