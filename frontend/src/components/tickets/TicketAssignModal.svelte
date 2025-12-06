<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";
  import FormInput from "../ui/FormInput.svelte";
  import Button from "../ui/Button.svelte";

  export let open = false;
  export let ticketId: string;

  const dispatch = createEventDispatcher();

  let contractorId = "";
  let submitting = false;

  async function handleSubmit() {
    if (!contractorId || !contractorId.trim()) {
      toast.error("Please enter a contractor ID");
      return;
    }

    try {
      submitting = true;
      dispatch("assigned", { contractorId: contractorId.trim() });
      handleClose();
    } catch (err: any) {
      toast.error(err.message || "Failed to assign ticket");
    } finally {
      submitting = false;
    }
  }

  function handleClose() {
    open = false;
    contractorId = "";
    dispatch("close");
  }
</script>

<Modal {open} on:close={handleClose} title="Assign Ticket to Contractor">
  <form on:submit|preventDefault={handleSubmit}>
    <div class="space-y-4">
      <p class="text-sm text-gray-600">
        Enter the contractor's user ID to assign this ticket. The contractor
        will be notified and can start work.
      </p>

      <FormInput
        label="Contractor ID"
        bind:value={contractorId}
        required
        placeholder="Enter contractor user ID"
      />

      <!-- Note: In a real implementation, this would be a searchable dropdown
           showing available contractors with their names, skills, ratings, etc. -->
      <p class="text-xs text-gray-500">
        💡 Tip: In the future, this will be a searchable dropdown of available
        contractors with their specialties and ratings.
      </p>
    </div>

    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose}>
        Cancel
      </Button>
      <Button type="submit" loading={submitting}>
        Assign Ticket
      </Button>
    </div>
  </form>
</Modal>
