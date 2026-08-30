<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "svelte-i18n";
  import { ResourceType } from "../../lib/api/bookings";

  let { type }: { type: ResourceType } = $props();

  const typeConfig: Record<
    ResourceType,
    { class: string; icon: string }
  > = {
    [ResourceType.MeetingRoom]: {
      class: "bg-blue-100 text-blue-800",
      icon: "💼",
    },
    [ResourceType.PartyRoom]: {
      class: "bg-pink-100 text-pink-800",
      icon: "🎉",
    },
    [ResourceType.Gym]: {
      class: "bg-red-100 text-red-800",
      icon: "🏋️",
    },
    [ResourceType.SwimmingPool]: {
      class: "bg-cyan-100 text-cyan-800",
      icon: "🏊",
    },
    [ResourceType.Sauna]: {
      class: "bg-orange-100 text-orange-800",
      icon: "🧖",
    },
    [ResourceType.ParkingSpace]: {
      class: "bg-gray-100 text-gray-800",
      icon: "🚗",
    },
    [ResourceType.GuestRoom]: {
      class: "bg-purple-100 text-purple-800",
      icon: "🛏️",
    },
    [ResourceType.Rooftop]: {
      class: "bg-yellow-100 text-yellow-800",
      icon: "🌆",
    },
    [ResourceType.Garden]: {
      class: "bg-green-100 text-green-800",
      icon: "🌳",
    },
    [ResourceType.LaundryRoom]: {
      class: "bg-teal-100 text-teal-800",
      icon: "🧺",
    },
    [ResourceType.StorageSpace]: {
      class: "bg-amber-100 text-amber-800",
      icon: "📦",
    },
    [ResourceType.CoworkingSpace]: {
      class: "bg-indigo-100 text-indigo-800",
      icon: "💻",
    },
    [ResourceType.Other]: {
      class: "bg-gray-100 text-gray-700",
      icon: "🏢",
    },
  };

  let config = $derived(typeConfig[type]);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {$_(`bookings.resourceType.${type}`)}
</span>
