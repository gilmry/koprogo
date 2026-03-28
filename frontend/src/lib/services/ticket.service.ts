import { ticketsApi, TicketStatus, type Ticket } from "../api/tickets";

/** Actions that can be performed on a ticket */
export type TicketAction =
  | "assign"
  | "start"
  | "resolve"
  | "close"
  | "cancel"
  | "reopen"
  | "delete";

/**
 * Determine which actions are available for a ticket based on its status and user role.
 * Extracts the status machine logic from TicketDetail.svelte template.
 */
export function getAvailableActions(
  ticket: Ticket,
  canManage: boolean,
  isContractor: boolean,
): TicketAction[] {
  const actions: TicketAction[] = [];

  switch (ticket.status) {
    case TicketStatus.Open:
      if (canManage) actions.push("assign", "cancel");
      break;
    case TicketStatus.Assigned:
      if (isContractor) actions.push("start");
      if (canManage) actions.push("cancel");
      break;
    case TicketStatus.InProgress:
      if (isContractor) actions.push("resolve");
      break;
    case TicketStatus.Resolved:
      if (canManage) actions.push("close");
      break;
    case TicketStatus.Closed:
    case TicketStatus.Cancelled:
      actions.push("reopen");
      break;
  }

  if (canManage) actions.push("delete");

  return actions;
}

/**
 * Load tickets based on view type.
 * Consolidates the 3-way switch from TicketList.svelte.
 */
export async function loadTickets(
  view: "all" | "my" | "assigned",
  buildingId?: string,
): Promise<Ticket[]> {
  switch (view) {
    case "my":
      return ticketsApi.listMy();
    case "assigned":
      return ticketsApi.listAssigned();
    default:
      if (buildingId) {
        return ticketsApi.listByBuilding(buildingId);
      }
      return [];
  }
}

/**
 * Execute a ticket status transition.
 * Single entry point for assign/start/resolve/close/cancel/reopen.
 */
export async function transitionTicket(
  ticketId: string,
  action: Exclude<TicketAction, "delete">,
  payload?: { contractorId?: string },
): Promise<Ticket> {
  switch (action) {
    case "assign":
      if (!payload?.contractorId)
        throw new Error("contractorId required for assign");
      return ticketsApi.assign(ticketId, payload.contractorId);
    case "start":
      return ticketsApi.start(ticketId);
    case "resolve":
      return ticketsApi.resolve(ticketId);
    case "close":
      return ticketsApi.close(ticketId);
    case "cancel":
      return ticketsApi.cancel(ticketId);
    case "reopen":
      return ticketsApi.reopen(ticketId);
  }
}
