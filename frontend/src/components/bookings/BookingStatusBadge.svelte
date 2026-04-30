<script lang="ts">
  // Svelte 5 runes mode
  import { BookingStatus } from "../../lib/api/bookings";

  let { status }: { status: BookingStatus } = $props();

  const statusConfig: Record<BookingStatus, { label: string; class: string }> = {
    [BookingStatus.Pending]: {
      label: "Pending",
      class: "bg-yellow-100 text-yellow-800",
    },
    [BookingStatus.Approved]: {
      label: "Approved",
      class: "bg-green-100 text-green-800",
    },
    [BookingStatus.Active]: {
      label: "Active",
      class: "bg-blue-100 text-blue-800",
    },
    [BookingStatus.Completed]: {
      label: "Completed",
      class: "bg-gray-100 text-gray-800",
    },
    [BookingStatus.Cancelled]: {
      label: "Cancelled",
      class: "bg-gray-100 text-gray-600",
    },
    [BookingStatus.Rejected]: {
      label: "Rejected",
      class: "bg-red-100 text-red-800",
    },
  };

  let config = $derived(statusConfig[status]);
</script>

<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {config.class}">
  {config.label}
</span>
