<script lang="ts">
  import { SkillCategory } from "../../lib/api/skills";

  export let category: SkillCategory;

  const categoryConfig: Record<
    SkillCategory,
    { label: string; class: string; icon: string }
  > = {
    [SkillCategory.HomeRepair]: {
      label: "Home Repair",
      class: "bg-orange-100 text-orange-800",
      icon: "🔧",
    },
    [SkillCategory.Languages]: {
      label: "Languages",
      class: "bg-blue-100 text-blue-800",
      icon: "🗣️",
    },
    [SkillCategory.Technology]: {
      label: "Technology",
      class: "bg-indigo-100 text-indigo-800",
      icon: "💻",
    },
    [SkillCategory.Education]: {
      label: "Education",
      class: "bg-purple-100 text-purple-800",
      icon: "📚",
    },
    [SkillCategory.Arts]: {
      label: "Arts",
      class: "bg-fuchsia-100 text-fuchsia-800",
      icon: "🎨",
    },
    [SkillCategory.Sports]: {
      label: "Sports",
      class: "bg-cyan-100 text-cyan-800",
      icon: "⚽",
    },
    [SkillCategory.Cooking]: {
      label: "Cooking",
      class: "bg-red-100 text-red-800",
      icon: "🍳",
    },
    [SkillCategory.Gardening]: {
      label: "Gardening",
      class: "bg-green-100 text-green-800",
      icon: "🌱",
    },
    [SkillCategory.Health]: {
      label: "Health",
      class: "bg-rose-100 text-rose-800",
      icon: "💊",
    },
    [SkillCategory.Legal]: {
      label: "Legal",
      class: "bg-slate-100 text-slate-800",
      icon: "⚖️",
    },
    [SkillCategory.Financial]: {
      label: "Financial",
      class: "bg-emerald-100 text-emerald-800",
      icon: "💰",
    },
    [SkillCategory.PetCare]: {
      label: "Pet Care",
      class: "bg-yellow-100 text-yellow-800",
      icon: "🐕",
    },
    [SkillCategory.Other]: {
      label: "Other",
      class: "bg-gray-100 text-gray-800",
      icon: "✨",
    },
  };

  $: config = categoryConfig[category] || categoryConfig[SkillCategory.Other];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
