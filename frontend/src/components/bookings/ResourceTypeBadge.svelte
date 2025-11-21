<script lang="ts">
  import { ResourceType } from "../../lib/api/bookings";

  export let type: ResourceType;

  const typeConfig: Record<
    ResourceType,
    { label: string; class: string; icon: string }
  > = {
    [ResourceType.MeetingRoom]: {
      label: "Meeting Room",
      class: "bg-blue-100 text-blue-800",
      icon: "💼",
    },
    [ResourceType.PartyRoom]: {
      label: "Party Room",
      class: "bg-pink-100 text-pink-800",
      icon: "🎉",
    },
    [ResourceType.Gym]: {
      label: "Gym",
      class: "bg-red-100 text-red-800",
      icon: "🏋️",
    },
    [ResourceType.SwimmingPool]: {
      label: "Swimming Pool",
      class: "bg-cyan-100 text-cyan-800",
      icon: "🏊",
    },
    [ResourceType.Sauna]: {
      label: "Sauna",
      class: "bg-orange-100 text-orange-800",
      icon: "🧖",
    },
    [ResourceType.ParkingSpace]: {
      label: "Parking Space",
      class: "bg-gray-100 text-gray-800",
      icon: "🚗",
    },
    [ResourceType.GuestRoom]: {
      label: "Guest Room",
      class: "bg-purple-100 text-purple-800",
      icon: "🛏️",
    },
    [ResourceType.Rooftop]: {
      label: "Rooftop",
      class: "bg-yellow-100 text-yellow-800",
      icon: "🌆",
    },
    [ResourceType.Garden]: {
      label: "Garden",
      class: "bg-green-100 text-green-800",
      icon: "🌳",
    },
    [ResourceType.LaundryRoom]: {
      label: "Laundry Room",
      class: "bg-teal-100 text-teal-800",
      icon: "🧺",
    },
    [ResourceType.StorageSpace]: {
      label: "Storage Space",
      class: "bg-amber-100 text-amber-800",
      icon: "📦",
    },
    [ResourceType.CoworkingSpace]: {
      label: "Coworking Space",
      class: "bg-indigo-100 text-indigo-800",
      icon: "💻",
    },
    [ResourceType.Other]: {
      label: "Other",
      class: "bg-gray-100 text-gray-700",
      icon: "🏢",
    },
  };

  $: config = typeConfig[type];
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  <span class="mr-1">{config.icon}</span>
  {config.label}
</span>
