<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    ticketsApi,
    TicketStatus,
    type Ticket,
  } from "../../lib/api/tickets";
  import { formatDateTime } from "../../lib/utils/date.utils";
  import { isOverdue as checkOverdue } from "../../lib/utils/date.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import TicketStatusBadge from "./TicketStatusBadge.svelte";
  import TicketPriorityBadge from "./TicketPriorityBadge.svelte";
  import TicketAssignModal from "./TicketAssignModal.svelte";
  import Button from "../ui/Button.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";

  export let ticket: Ticket;
  export let canManage = false;
  export let isContractor = false;

  const dispatch = createEventDispatcher();

  let showAssignModal = false;
  let showDeleteConfirm = false;
  let actionLoading = false;

  async function handleAssign(event: CustomEvent) {
    const contractorId = event.detail.contractorId;
    const result = await withErrorHandling({
      action: () => ticketsApi.assign(ticket.id, contractorId),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.assigned_successfully"),
      errorMessage: $_("tickets.assign_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleStart() {
    const result = await withErrorHandling({
      action: () => ticketsApi.start(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.work_started"),
      errorMessage: $_("tickets.start_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleResolve() {
    const result = await withErrorHandling({
      action: () => ticketsApi.resolve(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.marked_resolved"),
      errorMessage: $_("tickets.resolve_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleClose() {
    const result = await withErrorHandling({
      action: () => ticketsApi.close(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.closed"),
      errorMessage: $_("tickets.close_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleCancel() {
    const result = await withErrorHandling({
      action: () => ticketsApi.cancel(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.cancelled"),
      errorMessage: $_("tickets.cancel_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleReopen() {
    const result = await withErrorHandling({
      action: () => ticketsApi.reopen(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.reopened"),
      errorMessage: $_("tickets.reopen_failed"),
    });
    if (result) { ticket = result; dispatch("updated", ticket); }
  }

  async function handleDelete() {
    const result = await withErrorHandling({
      action: () => ticketsApi.delete(ticket.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_("tickets.deleted"),
      errorMessage: $_("tickets.delete_failed"),
    });
    showDeleteConfirm = false;
    if (result !== undefined) dispatch("deleted", ticket.id);
  }
</script>

<div class="bg-white shadow rounded-lg overflow-hidden" data-testid="ticket-detail">
  <!-- Header -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-start justify-between">
      <div class="flex-1">
        <div class="flex items-center space-x-3 mb-2">
          <h1 class="text-2xl font-bold text-gray-900" data-testid="ticket-detail-title">{ticket.title}</h1>
          {#if checkOverdue(ticket.due_date, ticket.status)}
            <span
              class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800"
              data-testid="ticket-overdue-badge"
            >
              ⚠️ {$_("tickets.overdue")}
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
          <Button on:click={() => (showAssignModal = true)} size="sm" data-testid="ticket-assign-btn">
            {$_("tickets.assign_to_contractor")}
          </Button>
        {/if}

        {#if isContractor && ticket.status === TicketStatus.Assigned}
          <Button on:click={handleStart} loading={actionLoading} size="sm" data-testid="ticket-start-btn">
            {$_("tickets.start_work")}
          </Button>
        {/if}

        {#if isContractor && ticket.status === TicketStatus.InProgress}
          <Button on:click={handleResolve} loading={actionLoading} size="sm" data-testid="ticket-resolve-btn">
            {$_("tickets.mark_resolved")}
          </Button>
        {/if}

        {#if canManage && ticket.status === TicketStatus.Resolved}
          <Button on:click={handleClose} loading={actionLoading} size="sm" data-testid="ticket-close-btn">
            {$_("tickets.close_ticket")}
          </Button>
        {/if}

        {#if canManage && (ticket.status === TicketStatus.Open || ticket.status === TicketStatus.Assigned)}
          <Button
            on:click={handleCancel}
            loading={actionLoading}
            variant="outline"
            size="sm"
            data-testid="ticket-cancel-btn"
          >
            {$_("common.cancel")}
          </Button>
        {/if}

        {#if ticket.status === TicketStatus.Closed || ticket.status === TicketStatus.Cancelled}
          <Button
            on:click={handleReopen}
            loading={actionLoading}
            variant="outline"
            size="sm"
            data-testid="ticket-reopen-btn"
          >
            {$_("tickets.reopen")}
          </Button>
        {/if}

        {#if canManage}
          <Button
            on:click={() => (showDeleteConfirm = true)}
            variant="outline"
            size="sm"
            data-testid="ticket-delete-btn"
            class="text-red-600 hover:text-red-700"
          >
            {$_("common.delete")}
          </Button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Details -->
  <div class="px-6 py-4 space-y-6">
    <!-- Description -->
    <div>
      <h2 class="text-lg font-semibold text-gray-900 mb-2">{$_("tickets.description")}</h2>
      <p class="text-gray-700 whitespace-pre-wrap" data-testid="ticket-detail-description">{ticket.description}</p>
    </div>

    <!-- Metadata Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6" data-testid="ticket-detail-metadata">
      <!-- Left column -->
      <div class="space-y-4">
        <div>
          <dt class="text-sm font-medium text-gray-500">{$_("tickets.category")}</dt>
          <dd class="mt-1 text-sm text-gray-900">{ticket.category}</dd>
        </div>

        <div>
          <dt class="text-sm font-medium text-gray-500">{$_("tickets.requester")}</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {ticket.requester_name || $_("common.unknown")}
          </dd>
        </div>

        {#if ticket.unit_number}
          <div>
            <dt class="text-sm font-medium text-gray-500">{$_("tickets.unit")}</dt>
            <dd class="mt-1 text-sm text-gray-900">{ticket.unit_number}</dd>
          </div>
        {/if}

        <div>
          <dt class="text-sm font-medium text-gray-500">{$_("tickets.created_at")}</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {formatDateTime(ticket.created_at)}
          </dd>
        </div>
      </div>

      <!-- Right column -->
      <div class="space-y-4">
        <div>
          <dt class="text-sm font-medium text-gray-500">
            {$_("tickets.assigned_contractor")}
          </dt>
          <dd class="mt-1 text-sm text-gray-900">
            {ticket.assigned_contractor_name || $_("tickets.not_assigned")}
          </dd>
        </div>

        {#if ticket.due_date}
          <div>
            <dt class="text-sm font-medium text-gray-500">{$_("tickets.due_date")}</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDateTime(ticket.due_date)}
            </dd>
          </div>
        {/if}

        {#if ticket.resolved_at}
          <div>
            <dt class="text-sm font-medium text-gray-500">{$_("tickets.resolved_at")}</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDateTime(ticket.resolved_at)}
            </dd>
          </div>
        {/if}

        {#if ticket.closed_at}
          <div>
            <dt class="text-sm font-medium text-gray-500">{$_("tickets.closed_at")}</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDateTime(ticket.closed_at)}
            </dd>
          </div>
        {/if}

        <div>
          <dt class="text-sm font-medium text-gray-500">{$_("tickets.last_updated")}</dt>
          <dd class="mt-1 text-sm text-gray-900">
            {formatDateTime(ticket.updated_at)}
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
  isOpen={showDeleteConfirm}
  title={$_("tickets.delete_ticket")}
  message={$_("tickets.delete_confirmation")}
  confirmText={$_("common.delete")}
  variant="danger"
  on:confirm={handleDelete}
  on:cancel={() => (showDeleteConfirm = false)}
/>
