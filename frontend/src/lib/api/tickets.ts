import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Ticket API Client
 * Wraps all 17 backend endpoints for ticket management.
 *
 * Enums are re-exported from auto-generated api.d.ts (STORY-P7-103) —
 * TypeScript will refuse any value that doesn't exist in the Rust enum.
 */

/**
 * CreateTicketDto — superset des champs FE.
 *
 * Story B5 (FR31 / Phase B FE) — extensions optionnelles pour Complaint :
 *   - kind                  : "request" (par défaut) | "complaint"
 *   - severity              : low|normal|high|critical (requis si Complaint)
 *   - incident_date         : ISO 8601 (date passée typiquement)
 *   - evidence_attachments  : URLs publiques S3/MinIO (max 10)
 *   - witnesses             : user_ids des copropriétaires témoins (max 10)
 *
 * Rétro-compat : tous ces champs sont optionnels — un appel "Request" reste
 * strictement identique à avant (cf. mission : tests existants verts).
 */
export interface CreateTicketDto {
  building_id: string;
  title: string;
  description: string;
  priority: TicketPriority;
  category: TicketCategory;
  requester_id?: string; // Ignoré par le backend (utilise le JWT), gardé pour compat frontend
  unit_id?: string;
  // Story B5 extensions (optionnels — rétro-compat Request)
  kind?: TicketKind;
  severity?: TicketSeverity;
  incident_date?: string;
  evidence_attachments?: string[];
  witnesses?: string[];
}

export interface Ticket {
  id: string;
  organization_id: string;
  building_id: string;
  title: string;
  description: string;
  status: TicketStatus;
  priority: TicketPriority;
  category: TicketCategory;
  created_by: string;
  requester_name?: string;
  assigned_to?: string;
  assigned_to_name?: string;
  unit_id?: string;
  unit_number?: string;
  due_date?: string;
  resolved_at?: string;
  closed_at?: string;
  created_at: string;
  updated_at: string;
}

// Re-exported from generated api.d.ts — single source of truth.
// Backend TicketStatus has 5 variants (no Assigned — the "assigned" state
// is derived from `assigned_to IS NOT NULL` + Open/InProgress).
export type TicketStatus = components["schemas"]["TicketStatus"];
export const TicketStatus = {
  Open: "Open" as const,
  InProgress: "InProgress" as const,
  Resolved: "Resolved" as const,
  Closed: "Closed" as const,
  Cancelled: "Cancelled" as const,
} satisfies Record<string, TicketStatus>;

export type TicketPriority = components["schemas"]["TicketPriority"];
export const TicketPriority = {
  Low: "Low" as const, // 7 days
  Medium: "Medium" as const, // 3 days
  High: "High" as const, // 24h
  Critical: "Critical" as const, // 1h (also covers "urgent")
} satisfies Record<string, TicketPriority>;

export type TicketCategory = components["schemas"]["TicketCategory"];
export const TicketCategory = {
  Plumbing: "Plumbing" as const,
  Electrical: "Electrical" as const,
  Heating: "Heating" as const,
  CommonAreas: "CommonAreas" as const,
  Elevator: "Elevator" as const,
  Security: "Security" as const,
  Cleaning: "Cleaning" as const,
  Landscaping: "Landscaping" as const,
  Other: "Other" as const,
} satisfies Record<string, TicketCategory>;

// Story B5 (Phase B FE) — TicketKind / TicketSeverity re-exports.
// Source de vérité = api.d.ts généré depuis OpenAPI (Story B0).
export type TicketKind = components["schemas"]["TicketKind"];
export const TicketKind = {
  Request: "request" as const,
  Complaint: "complaint" as const,
} satisfies Record<string, TicketKind>;

export type TicketSeverity = components["schemas"]["TicketSeverity"];
export const TicketSeverity = {
  Low: "low" as const,
  Normal: "normal" as const,
  High: "high" as const,
  Critical: "critical" as const,
} satisfies Record<string, TicketSeverity>;

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
    // Envoyer uniquement les champs attendus par le backend (CreateTicketRequest)
    // Le backend ignore requester_id et utilise le user_id du JWT
    const payload: Record<string, any> = {
      building_id: data.building_id,
      title: data.title,
      description: data.description,
      priority: data.priority,
      category: data.category,
    };
    // unit_id seulement si c'est un UUID valide (pas vide)
    if (data.unit_id && data.unit_id.trim() !== "") {
      payload.unit_id = data.unit_id;
    }
    // Story B5 — Complaint extensions (optionnels, rétro-compat Request).
    if (data.kind) payload.kind = data.kind;
    if (data.severity) payload.severity = data.severity;
    if (data.incident_date) payload.incident_date = data.incident_date;
    if (data.evidence_attachments && data.evidence_attachments.length > 0) {
      payload.evidence_attachments = data.evidence_attachments;
    }
    if (data.witnesses && data.witnesses.length > 0) {
      payload.witnesses = data.witnesses;
    }
    return api.post("/tickets", payload);
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
    return api.put(`/tickets/${id}/assign`, { assigned_to: contractorId });
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
