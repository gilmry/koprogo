<script lang="ts">
  // Svelte 5 runes mode
  import { type BookableResource } from "../../lib/api/bookings";
  import { _ } from "../../lib/i18n";
  import BookingCreateModal from "./BookingCreateModal.svelte";

  let {
    resource,
    ownerId,
  }: {
    resource: BookableResource;
    ownerId: string;
  } = $props();

  let modalOpen = $state(false);
</script>

{#if resource.status === "Available"}
  <button
    data-testid="booking-panel-open-button"
    onclick={() => (modalOpen = true)}
    class="w-full bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 font-medium"
  >
    {$_("bookings.bookThis")}
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
    <!-- Le message était en anglais ET affichait la valeur brute du statut
         (#792, #774). Le copropriétaire lisait « This resource is currently
         unavailable (Maintenance) ». -->
    <p class="text-red-800">{$_("bookings.resourceUnavailable")}</p>
  </div>
{/if}
