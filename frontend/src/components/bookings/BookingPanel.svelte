<script lang="ts">
  // Svelte 5 runes mode
  import { type BookableResource } from "../../lib/api/bookings";
  import BookingCreateModal from "./BookingCreateModal.svelte";

  let { resource, ownerId }: {
    resource: BookableResource;
    ownerId: string;
  } = $props();

  let modalOpen = $state(false);
</script>

{#if resource.status === "Available"}
  <button
    onclick={() => (modalOpen = true)}
    class="w-full bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 font-medium"
  >
    Book This Resource
  </button>

  <BookingCreateModal
    isOpen={modalOpen}
    {resource}
    {ownerId}
    onclose={() => (modalOpen = false)}
    oncreated={() => (modalOpen = false)}
  />
{:else}
  <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-center">
    <p class="text-red-800">This resource is currently unavailable ({resource.status})</p>
  </div>
{/if}
