<script lang="ts">
  import { ObjectCategory } from "../../lib/api/sharing";

  export let category: ObjectCategory;

  const categoryConfig: Record<
    ObjectCategory,
    { label: string; class: string; icon: string }
  > = {
    [ObjectCategory.Tools]: {
      label: "Tools",
      class: "bg-orange-100 text-orange-800",
      icon: "🔧",
    },
    [ObjectCategory.GardenEquipment]: {
      label: "Garden Equipment",
      class: "bg-green-100 text-green-800",
      icon: "🌿",
    },
    [ObjectCategory.KitchenAppliances]: {
      label: "Kitchen Appliances",
      class: "bg-red-100 text-red-800",
      icon: "🍳",
    },
    [ObjectCategory.Electronics]: {
      label: "Electronics",
      class: "bg-blue-100 text-blue-800",
      icon: "📱",
    },
    [ObjectCategory.Sports]: {
      label: "Sports",
      class: "bg-cyan-100 text-cyan-800",
      icon: "⚽",
    },
    [ObjectCategory.Books]: {
      label: "Books",
      class: "bg-purple-100 text-purple-800",
      icon: "📚",
    },
    [ObjectCategory.ChildrenEquipment]: {
      label: "Children Equipment",
      class: "bg-pink-100 text-pink-800",
      icon: "🧸",
    },
    [ObjectCategory.PartySupplies]: {
      label: "Party Supplies",
      class: "bg-fuchsia-100 text-fuchsia-800",
      icon: "🎉",
    },
    [ObjectCategory.Other]: {
      label: "Other",
      class: "bg-gray-100 text-gray-800",
      icon: "📦",
    },
  };

  $: config = categoryConfig[category];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
