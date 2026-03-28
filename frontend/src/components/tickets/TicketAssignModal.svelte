<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import { toast } from "../../stores/toast";
  import { withErrorHandling } from "../../lib/utils/error.utils";
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
      toast.error($_("tickets.enter_contractor_id"));
      return;
    }

    await withErrorHandling({
      action: async () => {
        dispatch("assigned", { contractorId: contractorId.trim() });
      },
      setLoading: (v) => submitting = v,
      errorMessage: $_("tickets.assign_failed"),
    });
    handleClose();
  }

  function handleClose() {
    open = false;
    contractorId = "";
    dispatch("close");
  }
</script>

<Modal isOpen={open} on:close={handleClose} title={$_("tickets.assign_to_contractor")}>
  <form on:submit|preventDefault={handleSubmit} data-testid="ticket-assign-form">
    <div class="space-y-4">
      <p class="text-sm text-gray-600" data-testid="ticket-assign-description">
        {$_("tickets.assign_description")}
      </p>

      <FormInput
        label={$_("tickets.contractor_id")}
        bind:value={contractorId}
        required
        placeholder={$_("tickets.contractor_id_placeholder")}
        data-testid="ticket-contractor-input"
      />

      <!-- Note: In a real implementation, this would be a searchable dropdown
           showing available contractors with their names, skills, ratings, etc. -->
      <p class="text-xs text-gray-500" data-testid="ticket-assign-tip">
        {$_("tickets.assign_tip")}
      </p>
    </div>

    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose} data-testid="ticket-assign-cancel-btn">
        {$_("common.cancel")}
      </Button>
      <Button type="submit" loading={submitting} data-testid="ticket-assign-submit-btn">
        {$_("tickets.assign_ticket")}
      </Button>
    </div>
  </form>
</Modal>
