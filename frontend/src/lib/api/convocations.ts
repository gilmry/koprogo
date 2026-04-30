import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Convocation API Client
 * Wraps all 14 backend endpoints for automatic AG invitations
 * Belgian legal deadlines: Ordinary 15 days, Extraordinary 8 days
 *
 * Enums are re-exported from auto-generated api.d.ts (STORY-P7-704) —
 * TypeScript will refuse any value that doesn't exist in the Rust enum.
 */

export interface Convocation {
  id: string;
  meeting_id: string;
  building_id: string;
  organization_id: string;
  meeting_type: MeetingType;
  meeting_date: string;
  minimum_send_date: string;
  status: ConvocationStatus;
  pdf_file_path?: string;
  language: string;
  total_recipients: number;
  opened_count: number;
  will_attend_count: number;
  respects_legal_deadline: boolean;
  created_at: string;
  updated_at: string;
}

// Hand-written: api.d.ts MeetingType is only "Ordinary"|"Extraordinary";
// SecondConvocation is needed by UI and exists in backend ConvocationType schema
// but not in MeetingType schema. Kept as hand-written enum until backend aligns.
export enum MeetingType {
  Ordinary = "Ordinary",
  Extraordinary = "Extraordinary",
  SecondConvocation = "SecondConvocation",
}

// Re-exported from generated api.d.ts — single source of truth.
export type ConvocationStatus = components["schemas"]["ConvocationStatus"];
export const ConvocationStatus = {
  Draft: "Draft" as const,
  Scheduled: "Scheduled" as const,
  Sent: "Sent" as const,
  Cancelled: "Cancelled" as const,
} satisfies Record<string, ConvocationStatus>;

export type AttendanceStatus = components["schemas"]["AttendanceStatus"];
export const AttendanceStatus = {
  Pending: "Pending" as const,
  WillAttend: "WillAttend" as const,
  WillNotAttend: "WillNotAttend" as const,
  Attended: "Attended" as const,
  DidNotAttend: "DidNotAttend" as const,
} satisfies Record<string, AttendanceStatus>;

export interface ConvocationRecipient {
  id: string;
  convocation_id: string;
  owner_id: string;
  owner_name?: string;
  owner_email: string;
  email_sent_at?: string;
  email_opened_at?: string;
  email_failed: boolean;
  reminder_sent_at?: string;
  attendance_status: AttendanceStatus;
  proxy_owner_id?: string;
  proxy_owner_name?: string;
  needs_reminder: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateConvocationDto {
  meeting_id: string;
  building_id: string;
  meeting_type: MeetingType;
  meeting_date: string;
  language?: string;
}

export interface TrackingSummary {
  total_recipients: number;
  email_sent: number;
  email_opened: number;
  email_failed: number;
  will_attend: number;
  will_not_attend: number;
  attended: number;
  did_not_attend: number;
  pending: number;
  opening_rate: number;
  attendance_rate: number;
}

/**
 * Convocations API functions
 */
export const convocationsApi = {
  /**
   * Create convocation (validates legal deadline)
   */
  async create(data: CreateConvocationDto): Promise<Convocation> {
    return api.post("/convocations", data);
  },

  /**
   * Get convocation by ID
   */
  async getById(id: string): Promise<Convocation> {
    return api.get(`/convocations/${id}`);
  },

  /**
   * Get convocation by meeting ID
   */
  async getByMeetingId(meetingId: string): Promise<Convocation> {
    return api.get(`/convocations/meeting/${meetingId}`);
  },

  /**
   * List convocations by building
   */
  async listByBuilding(buildingId: string): Promise<Convocation[]> {
    return api.get(`/buildings/${buildingId}/convocations`);
  },

  /**
   * List convocations by organization
   */
  async listByOrganization(organizationId: string): Promise<Convocation[]> {
    return api.get(`/organizations/${organizationId}/convocations`);
  },

  /**
   * Schedule send date (validates before legal deadline)
   */
  async schedule(id: string, sendDate: string): Promise<Convocation> {
    return api.put(`/convocations/${id}/schedule`, { send_date: sendDate });
  },

  /**
   * Send convocation (generates PDF, creates recipients, triggers emails)
   */
  async send(id: string): Promise<Convocation> {
    return api.post(`/convocations/${id}/send`, {});
  },

  /**
   * Cancel convocation
   */
  async cancel(id: string): Promise<Convocation> {
    return api.put(`/convocations/${id}/cancel`, {});
  },

  /**
   * Delete convocation
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/convocations/${id}`);
  },

  /**
   * Get all recipients
   */
  async getRecipients(id: string): Promise<ConvocationRecipient[]> {
    return api.get(`/convocations/${id}/recipients`);
  },

  /**
   * Get tracking summary (opening rate, attendance rate)
   */
  async getTrackingSummary(id: string): Promise<TrackingSummary> {
    return api.get(`/convocations/${id}/tracking-summary`);
  },

  /**
   * Mark email opened (tracking pixel endpoint)
   */
  async markEmailOpened(recipientId: string): Promise<void> {
    return api.put(`/convocation-recipients/${recipientId}/email-opened`, {});
  },

  /**
   * Update attendance status
   */
  async updateAttendance(
    recipientId: string,
    status: AttendanceStatus,
  ): Promise<ConvocationRecipient> {
    return api.put(`/convocation-recipients/${recipientId}/attendance`, {
      status,
    });
  },

  /**
   * Set proxy delegation (Belgian "procuration")
   */
  async setProxy(
    recipientId: string,
    proxyOwnerId: string,
  ): Promise<ConvocationRecipient> {
    return api.put(`/convocation-recipients/${recipientId}/proxy`, {
      proxy_owner_id: proxyOwnerId,
    });
  },

  /**
   * Send J-3 reminders (3 days before meeting)
   */
  async sendReminders(id: string): Promise<void> {
    return api.post(`/convocations/${id}/reminders`, {});
  },
};
