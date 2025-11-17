<script lang="ts">
  import { ProficiencyLevel } from "../../lib/api/skills";

  export let level: ProficiencyLevel;

  const levelConfig: Record<
    ProficiencyLevel,
    { label: string; class: string; stars: string }
  > = {
    [ProficiencyLevel.Beginner]: {
      label: "Beginner",
      class: "bg-gray-100 text-gray-700",
      stars: "⭐",
    },
    [ProficiencyLevel.Intermediate]: {
      label: "Intermediate",
      class: "bg-blue-100 text-blue-700",
      stars: "⭐⭐",
    },
    [ProficiencyLevel.Advanced]: {
      label: "Advanced",
      class: "bg-green-100 text-green-700",
      stars: "⭐⭐⭐",
    },
    [ProficiencyLevel.Expert]: {
      label: "Expert",
      class: "bg-purple-100 text-purple-700",
      stars: "⭐⭐⭐⭐",
    },
    [ProficiencyLevel.Professional]: {
      label: "Professional",
      class: "bg-yellow-100 text-yellow-800",
      stars: "⭐⭐⭐⭐⭐",
    },
  };

  $: config = levelConfig[level];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.stars}</span>
  {config.label}
</span>
