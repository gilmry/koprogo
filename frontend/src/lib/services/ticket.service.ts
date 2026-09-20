import { ticketsApi, TicketStatus, type Ticket } from "../api/tickets";

/** Actions that can be performed on a ticket */
export type TicketAction =
  "assign" | "start" | "resolve" | "close" | "cancel" | "reopen" | "delete";

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

  // Backend domain has 5 statuses (no Assigned).
  // Assigning a ticket auto-transitions Open → InProgress.
  switch (ticket.status) {
    case TicketStatus.Open:
      if (canManage) actions.push("assign", "cancel");
      break;
    case TicketStatus.InProgress:
      if (isContractor || canManage) actions.push("resolve");
      if (canManage) actions.push("cancel");
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
  payload?: { contractorId?: string; motif?: string },
): Promise<Ticket> {
  switch (action) {
    case "assign":
      if (!payload?.contractorId)
        throw new Error("contractorId required for assign");
      return ticketsApi.assign(ticketId, payload.contractorId);
    case "start":
      return ticketsApi.start(ticketId);
    case "resolve":
      // Le motif est exigé ICI, pas remplacé par un défaut. Trois DTO du
      // backend portent un champ obligatoire, et l'appelant envoyait `{}` :
      // les trois transitions rendaient 400 et le bouton ne faisait rien
      // (#977). Un défaut silencieux aurait juste déplacé le mensonge du
      // serveur vers le registre.
      if (!payload?.motif)
        throw new Error("motif required for resolve (resolution_notes)");
      return ticketsApi.resolve(ticketId, payload.motif);
    case "close":
      return ticketsApi.close(ticketId);
    case "cancel":
      if (!payload?.motif) throw new Error("motif required for cancel");
      return ticketsApi.cancel(ticketId, payload.motif);
    case "reopen":
      if (!payload?.motif) throw new Error("motif required for reopen");
      return ticketsApi.reopen(ticketId, payload.motif);
  }
}
