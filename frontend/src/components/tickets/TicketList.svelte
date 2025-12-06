<script lang="ts">
  import { onMount } from "svelte";
  import {
    ticketsApi,
    TicketStatus,
    TicketPriority,
    TicketCategory,
    type Ticket,
  } from "../../lib/api/tickets";
  import TicketStatusBadge from "./TicketStatusBadge.svelte";
  import TicketPriorityBadge from "./TicketPriorityBadge.svelte";
  import { toast } from "../../stores/toast";

  export let buildingId: string | undefined = undefined;
  export let view: "all" | "my" | "assigned" = "all";

  let tickets: Ticket[] = [];
  let loading = true;
  let error = "";

  // Filters
  let statusFilter: TicketStatus | "all" = "all";
  let priorityFilter: TicketPriority | "all" = "all";
  let categoryFilter: TicketCategory | "all" = "all";
  let searchQuery = "";

  onMount(async () => {
    await loadTickets();
  });

  async function loadTickets() {
    try {
      loading = true;
      error = "";

      if (view === "my") {
        tickets = await ticketsApi.listMy();
      } else if (view === "assigned") {
        tickets = await ticketsApi.listAssigned();
      } else if (buildingId) {
        tickets = await ticketsApi.listByBuilding(buildingId);
      } else {
        // This would need organization_id from context
        tickets = [];
      }
    } catch (err: any) {
      error = err.message || "Failed to load tickets";
      toast.error(error);
    } finally {
      loading = false;
    }
  }

  $: filteredTickets = tickets.filter((ticket) => {
    if (statusFilter !== "all" && ticket.status !== statusFilter) return false;
    if (priorityFilter !== "all" && ticket.priority !== priorityFilter)
      return false;
    if (categoryFilter !== "all" && ticket.category !== categoryFilter)
      return false;
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        ticket.title.toLowerCase().includes(query) ||
        ticket.description.toLowerCase().includes(query) ||
        ticket.requester_name?.toLowerCase().includes(query) ||
        ticket.assigned_contractor_name?.toLowerCase().includes(query)
      );
    }
    return true;
  });

  function getTicketUrl(ticketId: string): string {
    return `/ticket-detail?id=${ticketId}`;
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("nl-BE", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function isOverdue(ticket: Ticket): boolean {
    if (!ticket.due_date || ticket.status === TicketStatus.Closed) return false;
    return new Date(ticket.due_date) < new Date();
  }
</script>

<div class="bg-white shadow rounded-lg">
  <!-- Header with filters -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-gray-900">
        {#if view === "my"}
          My Tickets
        {:else if view === "assigned"}
          Assigned Tickets
        {:else}
          All Tickets
        {/if}
        <span class="ml-2 text-sm text-gray-500">
          ({filteredTickets.length})
        </span>
      </h2>
      <button
        on:click={loadTickets}
        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
      >
        Refresh
      </button>
    </div>

    <!-- Filters -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <!-- Search -->
      <div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search tickets..."
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      <!-- Status filter -->
      <div>
        <select
          bind:value={statusFilter}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Statuses</option>
          <option value={TicketStatus.Open}>Open</option>
          <option value={TicketStatus.Assigned}>Assigned</option>
          <option value={TicketStatus.InProgress}>In Progress</option>
          <option value={TicketStatus.Resolved}>Resolved</option>
          <option value={TicketStatus.Closed}>Closed</option>
        </select>
      </div>

      <!-- Priority filter -->
      <div>
        <select
          bind:value={priorityFilter}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Priorities</option>
          <option value={TicketPriority.Critical}>Critical</option>
          <option value={TicketPriority.Urgent}>Urgent</option>
          <option value={TicketPriority.High}>High</option>
          <option value={TicketPriority.Medium}>Medium</option>
          <option value={TicketPriority.Low}>Low</option>
        </select>
      </div>

      <!-- Category filter -->
      <div>
        <select
          bind:value={categoryFilter}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Categories</option>
          <option value={TicketCategory.Plumbing}>Plumbing</option>
          <option value={TicketCategory.Electrical}>Electrical</option>
          <option value={TicketCategory.Heating}>Heating</option>
          <option value={TicketCategory.Cleaning}>Cleaning</option>
          <option value={TicketCategory.Security}>Security</option>
          <option value={TicketCategory.General}>General</option>
          <option value={TicketCategory.Emergency}>Emergency</option>
        </select>
      </div>
    </div>
  </div>

  <!-- Tickets list -->
  <div class="divide-y divide-gray-200">
    {#if loading}
      <div class="px-6 py-12 text-center text-gray-500">Loading tickets...</div>
    {:else if error}
      <div class="px-6 py-12 text-center text-red-600">{error}</div>
    {:else if filteredTickets.length === 0}
      <div class="px-6 py-12 text-center text-gray-500">
        No tickets found matching your filters.
      </div>
    {:else}
      {#each filteredTickets as ticket (ticket.id)}
        <a
          href={getTicketUrl(ticket.id)}
          class="block px-6 py-4 hover:bg-gray-50 transition-colors"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center space-x-2 mb-2">
                <h3 class="text-lg font-medium text-gray-900 truncate">
                  {ticket.title}
                </h3>
                {#if isOverdue(ticket)}
                  <span
                    class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800"
                  >
                    ⚠️ Overdue
                  </span>
                {/if}
              </div>

              <p class="text-sm text-gray-600 mb-2 line-clamp-2">
                {ticket.description}
              </p>

              <div class="flex items-center space-x-4 text-sm text-gray-500">
                <span>#{ticket.id.slice(0, 8)}</span>
                <span>Requester: {ticket.requester_name || "Unknown"}</span>
                {#if ticket.assigned_contractor_name}
                  <span>
                    Assigned to: {ticket.assigned_contractor_name}
                  </span>
                {/if}
                {#if ticket.unit_number}
                  <span>Unit: {ticket.unit_number}</span>
                {/if}
                <span>Category: {ticket.category}</span>
                {#if ticket.due_date}
                  <span>Due: {formatDate(ticket.due_date)}</span>
                {/if}
              </div>
            </div>

            <div class="flex flex-col items-end space-y-2 ml-4">
              <TicketStatusBadge status={ticket.status} />
              <TicketPriorityBadge priority={ticket.priority} />
              <span class="text-xs text-gray-500">
                {formatDate(ticket.created_at)}
              </span>
            </div>
          </div>
        </a>
      {/each}
    {/if}
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
