<script lang="ts">
  // Svelte 5 runes mode
  import { type SharedObject } from "../../lib/api/sharing";
  import LoanRequestModal from "./LoanRequestModal.svelte";

  let { object, borrowerId }: {
    object: SharedObject;
    borrowerId: string;
  } = $props();

  let modalOpen = $state(false);
</script>

{#if object.availability_status === "Available"}
  <button
    onclick={() => (modalOpen = true)}
    class="w-full bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 font-medium"
  >
    Request to Borrow
  </button>

  <LoanRequestModal
    isOpen={modalOpen}
    {object}
    {borrowerId}
    onclose={() => (modalOpen = false)}
    oncreated={() => (modalOpen = false)}
  />
{:else}
  <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 text-center">
    <p class="text-yellow-800">This object is currently unavailable</p>
  </div>
{/if}
