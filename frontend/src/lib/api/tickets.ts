import { api } from "../api";

/**
 * Ticket API Client
 * Wraps all 17 backend endpoints for ticket management
 */

export interface CreateTicketDto {
  building_id: string;
  title: string;
  description: string;
  priority: TicketPriority;
  category: TicketCategory;
  requester_id: string;
  unit_id?: string;
}

export interface Ticket {
  id: string;
  building_id: string;
  title: string;
  description: string;
  status: TicketStatus;
  priority: TicketPriority;
  category: TicketCategory;
  requester_id: string;
  requester_name?: string;
  assigned_contractor_id?: string;
  assigned_contractor_name?: string;
  unit_id?: string;
  unit_number?: string;
  due_date?: string;
  resolved_at?: string;
  closed_at?: string;
  created_at: string;
  updated_at: string;
}

export enum TicketStatus {
  Open = "Open",
  Assigned = "Assigned",
  InProgress = "InProgress",
  Resolved = "Resolved",
  Closed = "Closed",
  Cancelled = "Cancelled",
}

export enum TicketPriority {
  Low = "Low", // 7 days
  Medium = "Medium", // 3 days
  High = "High", // 24h
  Urgent = "Urgent", // 4h
  Critical = "Critical", // 1h
}

export enum TicketCategory {
  Plumbing = "Plumbing",
  Electrical = "Electrical",
  Heating = "Heating",
  Cleaning = "Cleaning",
  Security = "Security",
  General = "General",
  Emergency = "Emergency",
}

export interface TicketStatistics {
  total_tickets: number;
  open_tickets: number;
  assigned_tickets: number;
  in_progress_tickets: number;
  resolved_tickets: number;
  closed_tickets: number;
  cancelled_tickets: number;
  overdue_tickets: number;
  average_resolution_time_hours?: number;
}

/**
 * Tickets API functions
 */
export const ticketsApi = {
  /**
   * Create a new ticket
   */
  async create(data: CreateTicketDto): Promise<Ticket> {
    return api.post("/tickets", data);
  },

  /**
   * Get ticket by ID
   */
  async getById(id: string): Promise<Ticket> {
    return api.get(`/tickets/${id}`);
  },

  /**
   * List tickets by building
   */
  async listByBuilding(buildingId: string): Promise<Ticket[]> {
    return api.get(`/buildings/${buildingId}/tickets`);
  },

  /**
   * List tickets by organization
   */
  async listByOrganization(organizationId: string): Promise<Ticket[]> {
    return api.get(`/organizations/${organizationId}/tickets`);
  },

  /**
   * List my tickets (requester view)
   */
  async listMy(): Promise<Ticket[]> {
    return api.get("/tickets/my");
  },

  /**
   * List assigned tickets (contractor view)
   */
  async listAssigned(): Promise<Ticket[]> {
    return api.get("/tickets/assigned");
  },

  /**
   * List tickets by status
   */
  async listByStatus(status: TicketStatus): Promise<Ticket[]> {
    return api.get(`/tickets/status/${status}`);
  },

  /**
   * Assign ticket to contractor
   */
  async assign(id: string, contractorId: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/assign`, { contractor_id: contractorId });
  },

  /**
   * Start work on ticket
   */
  async start(id: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/start`, {});
  },

  /**
   * Mark ticket as resolved
   */
  async resolve(id: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/resolve`, {});
  },

  /**
   * Close ticket
   */
  async close(id: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/close`, {});
  },

  /**
   * Cancel ticket
   */
  async cancel(id: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/cancel`, {});
  },

  /**
   * Reopen ticket
   */
  async reopen(id: string): Promise<Ticket> {
    return api.put(`/tickets/${id}/reopen`, {});
  },

  /**
   * Delete ticket
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/tickets/${id}`);
  },

  /**
   * Get ticket statistics
   */
  async getStatistics(): Promise<TicketStatistics> {
    return api.get("/tickets/statistics");
  },

  /**
   * Get overdue tickets
   */
  async getOverdue(): Promise<Ticket[]> {
    return api.get("/tickets/overdue");
  },
};
