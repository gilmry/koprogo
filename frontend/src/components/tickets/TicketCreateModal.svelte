<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    ticketsApi,
    TicketPriority,
    TicketCategory,
    type CreateTicketDto,
  } from "../../lib/api/tickets";
  import { addToast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";
  import FormInput from "../ui/FormInput.svelte";
  import FormTextarea from "../ui/FormTextarea.svelte";
  import FormSelect from "../ui/FormSelect.svelte";
  import Button from "../ui/Button.svelte";

  export let open = false;
  export let buildingId: string;
  export let requesterId: string;
  export let unitId: string | undefined = undefined;

  const dispatch = createEventDispatcher();

  let formData: CreateTicketDto = {
    building_id: buildingId,
    title: "",
    description: "",
    priority: TicketPriority.Medium,
    category: TicketCategory.General,
    requester_id: requesterId,
    unit_id: unitId,
  };

  let submitting = false;
  let errors: Record<string, string> = {};

  function validate(): boolean {
    errors = {};

    if (!formData.title || formData.title.trim().length < 3) {
      errors.title = "Title must be at least 3 characters";
    }

    if (!formData.description || formData.description.trim().length < 10) {
      errors.description = "Description must be at least 10 characters";
    }

    return Object.keys(errors).length === 0;
  }

  async function handleSubmit() {
    if (!validate()) {
      addToast({
        message: "Please fix validation errors",
        type: "error",
      });
      return;
    }

    try {
      submitting = true;

      const ticket = await ticketsApi.create({
        ...formData,
        building_id: buildingId,
        requester_id: requesterId,
      });

      addToast({
        message: "Ticket created successfully",
        type: "success",
      });

      dispatch("created", ticket);
      handleClose();
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to create ticket",
        type: "error",
      });
    } finally {
      submitting = false;
    }
  }

  function handleClose() {
    open = false;
    formData = {
      building_id: buildingId,
      title: "",
      description: "",
      priority: TicketPriority.Medium,
      category: TicketCategory.General,
      requester_id: requesterId,
      unit_id: unitId,
    };
    errors = {};
    dispatch("close");
  }
</script>

<Modal {open} on:close={handleClose} title="Create Maintenance Ticket">
  <form on:submit|preventDefault={handleSubmit}>
    <div class="space-y-4">
      <!-- Title -->
      <FormInput
        label="Title"
        bind:value={formData.title}
        error={errors.title}
        required
        placeholder="Brief description of the issue"
      />

      <!-- Description -->
      <FormTextarea
        label="Description"
        bind:value={formData.description}
        error={errors.description}
        required
        rows={4}
        placeholder="Detailed description of the maintenance request..."
      />

      <!-- Priority -->
      <FormSelect
        label="Priority"
        bind:value={formData.priority}
        required
      >
        <option value={TicketPriority.Low}>Low (7 days)</option>
        <option value={TicketPriority.Medium}>Medium (3 days)</option>
        <option value={TicketPriority.High}>High (24 hours)</option>
        <option value={TicketPriority.Urgent}>Urgent (4 hours)</option>
        <option value={TicketPriority.Critical}>Critical (1 hour)</option>
      </FormSelect>

      <!-- Category -->
      <FormSelect
        label="Category"
        bind:value={formData.category}
        required
      >
        <option value={TicketCategory.General}>General</option>
        <option value={TicketCategory.Plumbing}>Plumbing</option>
        <option value={TicketCategory.Electrical}>Electrical</option>
        <option value={TicketCategory.Heating}>Heating</option>
        <option value={TicketCategory.Cleaning}>Cleaning</option>
        <option value={TicketCategory.Security}>Security</option>
        <option value={TicketCategory.Emergency}>Emergency</option>
      </FormSelect>

      <!-- Unit ID (optional) -->
      {#if !unitId}
        <FormInput
          label="Unit ID (Optional)"
          bind:value={formData.unit_id}
          placeholder="Unit where the issue is located"
        />
      {/if}
    </div>

    <!-- Actions -->
    <div class="mt-6 flex justify-end space-x-3">
      <Button type="button" variant="outline" on:click={handleClose}>
        Cancel
      </Button>
      <Button type="submit" loading={submitting}>
        Create Ticket
      </Button>
    </div>
  </form>
</Modal>
