<script lang="ts">
  import { type BookableResource } from "../../lib/api/bookings";
  import BookingCreateModal from "./BookingCreateModal.svelte";

  export let resource: BookableResource;
  export let ownerId: string;

  let modalOpen = false;
</script>

{#if resource.status === "Available"}
  <button
    on:click={() => (modalOpen = true)}
    class="w-full bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 font-medium"
  >
    Book This Resource
  </button>

  <BookingCreateModal
    isOpen={modalOpen}
    {resource}
    {ownerId}
    on:close={() => (modalOpen = false)}
    on:created={() => (modalOpen = false)}
  />
{:else}
  <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-center">
    <p class="text-red-800">This resource is currently unavailable ({resource.status})</p>
  </div>
{/if}
