<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    ticketsApi,
    TicketStatus,
    type Ticket,
  } from "../../lib/api/tickets";
  import { toast } from "../../stores/toast";
  import TicketStatusBadge from "./TicketStatusBadge.svelte";
  import TicketPriorityBadge from "./TicketPriorityBadge.svelte";
  import TicketAssignModal from "./TicketAssignModal.svelte";
  import Button from "../ui/Button.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";

  export let ticket: Ticket;
  export let canManage = false; // Syndic can manage
  export let isContractor = false; // Contractor can start/resolve

  const dispatch = createEventDispatcher();

  let showAssignModal = false;
  let showDeleteConfirm = false;
  let actionLoading = false;

  function formatDate(dateString: string | undefined): string {
    if (!dateString) return "N/A";
    return new Date(dateString).toLocaleDateString("nl-BE", {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function isOverdue(): boolean {
    if (!ticket.due_date || ticket.status === TicketStatus.Closed) return false;
    return new Date(ticket.due_date) < new Date();
  }

  async function handleAssign(event: CustomEvent) {
    const contractorId = event.detail.contractorId;
    try {
      actionLoading = true;
      ticket = await ticketsApi.assign(ticket.id, contractorId);
      toast.success("Ticket assigned successfully");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to assign ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleStart() {
    try {
      actionLoading = true;
      ticket = await ticketsApi.start(ticket.id);
      toast.success("Work started on ticket");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to start ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleResolve() {
    try {
      actionLoading = true;
      ticket = await ticketsApi.resolve(ticket.id);
      toast.success("Ticket marked as resolved");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to resolve ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleClose() {
    try {
      actionLoading = true;
      ticket = await ticketsApi.close(ticket.id);
      toast.success("Ticket closed");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to close ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleCancel() {
    try {
      actionLoading = true;
      ticket = await ticketsApi.cancel(ticket.id);
      toast.success("Ticket cancelled");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to cancel ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleReopen() {
    try {
      actionLoading = true;
      ticket = await ticketsApi.reopen(ticket.id);
      toast.success("Ticket reopened");
      dispatch("updated", ticket);
    } catch (err: any) {
      toast.error(err.message || "Failed to reopen ticket");
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    try {
      actionLoading = true;
      await ticketsApi.delete(ticket.id);
      toast.success("Ticket deleted");
      dispatch("deleted", ticket.id);
    } catch (err: any) {
      toast.error(err.message || "Failed to delete ticket");
    } finally {
      actionLoading = false;
      showDeleteConfirm = false;
    }
  }
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
  <!-- Header -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-start justify-between">
      <div class="flex-1">
        <div class="flex items-center space-x-3 mb-2">
          <h1 class="text-2xl font-bold text-gray-900">{ticket.title}</h1>
          {#if isOverdue()}
            <span
              class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800"
            >
              ⚠️ Overdue
            </span>
          {/if}
        </div>
        <div class="flex items-center space-x-3">
          <TicketStatusBadge status={ticket.status} />
          <TicketPriorityBadge priority={ticket.priority} />
          <span class="text-sm text-gray-500">
            ID: {ticket.id.slice(0, 8)}
          </span>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex flex-col space-y-2">
        {#if canManage && ticket.status === TicketStatus.Open}
          <Button on:click={() => (showAssignModal = true)} size="sm">
            Assign to Contractor
          </Button>
        {/if}

        {#if isContractor && ticket.status === TicketStatus.Assigned}
          <Button on:click={handleStart} loading={actionLoading} size="sm">
            Start Work
          </Button>
        {/if}

        {#if isContractor && ticket.status === TicketStatus.InProgress}
          <Button on:click={handleResolve} loading={actionLoading} size="sm">
            Mark as Resolved
          </Button>
        {/if}

        {#if canManage && ticket.status === TicketStatus.Resolved}
          <Button on:click={handleClose} loading={actionLoading} size="sm">
            Close Ticket
          </Button>
        {/if}

        {#if canManage && (ticket.status === TicketStatus.Open || ticket.status === TicketStatus.Assigned)}
          <Button
            on:click={handleCancel}
            loading={actionLoading}
            variant="outline"
            size="sm"
          >
            Cancel
          </Button>
        {/if}

        {#if ticket.status === TicketStatus.Closed || ticket.status === TicketStatus.Cancelled}
          <Button
            on:click={handleReopen}
            loading={actionLoading}
            variant="outline"
            size="sm"
          >
            Reopen
          </Button>
        {/if}

        {#if canManage}
          <Button
            on:click={() => (showDeleteConfirm = true)}
            variant="outline"
            size="sm"
            class="text-red-600 hover:text-red-700"
          >
            Delete
          </Button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Details -->
  <div class="px-6 py-4 space-y-6">
    <!-- Description -->
    <div>
      <h2 class="text-lg font-semibold text-gray-900 mb-2">Description</h2>
      <p class="text-gray-700 whitespace-pre-wrap">{ticket.description}</p>
    </div>

    <!-- Metadata Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <!-- Left column -->
      <div class="space-y-4">
        <div>
          <dt class="text-sm font-medium text-gray-500">Category</dt>
          <dd class="mt-1 text-sm text-gray-900">{ticket.category}</dd>
        </div>

        <div>
          <dt class="text-sm font-medium text-gray-500">Requester</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {ticket.requester_name || "Unknown"}
          </dd>
        </div>

        {#if ticket.unit_number}
          <div>
            <dt class="text-sm font-medium text-gray-500">Unit</dt>
            <dd class="mt-1 text-sm text-gray-900">{ticket.unit_number}</dd>
          </div>
        {/if}

        <div>
          <dt class="text-sm font-medium text-gray-500">Created At</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {formatDate(ticket.created_at)}
          </dd>
        </div>
      </div>

      <!-- Right column -->
      <div class="space-y-4">
        <div>
          <dt class="text-sm font-medium text-gray-500">
            Assigned Contractor
          </dt>
          <dd class="mt-1 text-sm text-gray-900">
            {ticket.assigned_contractor_name || "Not assigned"}
          </dd>
        </div>

        {#if ticket.due_date}
          <div>
            <dt class="text-sm font-medium text-gray-500">Due Date</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(ticket.due_date)}
            </dd>
          </div>
        {/if}

        {#if ticket.resolved_at}
          <div>
            <dt class="text-sm font-medium text-gray-500">Resolved At</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(ticket.resolved_at)}
            </dd>
          </div>
        {/if}

        {#if ticket.closed_at}
          <div>
            <dt class="text-sm font-medium text-gray-500">Closed At</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(ticket.closed_at)}
            </dd>
          </div>
        {/if}

        <div>
          <dt class="text-sm font-medium text-gray-500">Last Updated</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {formatDate(ticket.updated_at)}
          </dd>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Assign Modal -->
<TicketAssignModal
  bind:open={showAssignModal}
  ticketId={ticket.id}
  on:assigned={handleAssign}
/>

<!-- Delete Confirmation -->
<ConfirmDialog
  open={showDeleteConfirm}
  title="Delete Ticket"
  message="Are you sure you want to delete this ticket? This action cannot be undone."
  confirmText="Delete"
  confirmVariant="danger"
  on:confirm={handleDelete}
  on:cancel={() => (showDeleteConfirm = false)}
/>
