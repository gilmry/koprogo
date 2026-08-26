import { api } from "../api";

/**
 * AG Sessions API Client
 * Wraps all 9 backend endpoints for AG video session management
 * Belgian legal compliance: Art. 3.87 §1 CC — remote participation in general assemblies
 */

export type VideoPlatform =
  "Zoom" | "MicrosoftTeams" | "GoogleMeet" | "Jitsi" | "Whereby" | "Other";
export type AgSessionStatus = "Scheduled" | "Live" | "Ended" | "Cancelled";

export interface AgSession {
  id: string;
  meeting_id: string;
  organization_id: string;
  platform: VideoPlatform;
  video_url: string;
  host_url?: string;
  status: AgSessionStatus;
  remote_attendees_count: number;
  remote_voting_power: number;
  quorum_remote_contribution: number;
  access_password?: string;
  waiting_room_enabled: boolean;
  recording_enabled: boolean;
  started_at?: string;
  ended_at?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateAgSessionDto {
  platform: VideoPlatform;
  video_url: string;
  host_url?: string;
  access_password?: string;
  waiting_room_enabled?: boolean;
  recording_enabled?: boolean;
}

export interface RecordJoinDto {
  remote_voting_power: number;
}

export interface CombinedQuorumQuery {
  physical_quotas: number;
  total_building_quotas: number;
  /** Volet « têtes » du quorum double, Art. 3.87 §5 CC (#661). */
  physical_owners_count: number;
  total_owners_count: number;
}

export interface CombinedQuorumResponse {
  session_id: string;
  meeting_id: string;
  physical_quotas: number;
  remote_quotas: number;
  total_building_quotas: number;
  combined_percentage: number;
  physical_owners_count: number;
  remote_attendees_count: number;
  total_owners_count: number;
  /**
   * Art. 3.87 §5 CC — quorum DOUBLE : têtes > 50% ET quotités >= 50%, ou
   * quotités > 3/4. Décidé côté backend, jamais recalculé côté client.
   */
  quorum_reached: boolean;
}

export const agSessionsApi = {
  async createForMeeting(
    meetingId: string,
    data: CreateAgSessionDto,
  ): Promise<AgSession> {
    return api.post(`/meetings/${meetingId}/ag-session`, data);
  },

  async getByMeeting(meetingId: string): Promise<AgSession> {
    return api.get(`/meetings/${meetingId}/ag-session`);
  },

  async getById(id: string): Promise<AgSession> {
    return api.get(`/ag-sessions/${id}`);
  },

  async listAll(): Promise<AgSession[]> {
    return api.get("/ag-sessions");
  },

  async start(id: string): Promise<AgSession> {
    return api.put(`/ag-sessions/${id}/start`, {});
  },

  async end(id: string): Promise<AgSession> {
    return api.put(`/ag-sessions/${id}/end`, {});
  },

  async cancel(id: string): Promise<AgSession> {
    return api.put(`/ag-sessions/${id}/cancel`, {});
  },

  async recordJoin(id: string, data: RecordJoinDto): Promise<AgSession> {
    return api.put(`/ag-sessions/${id}/record-join`, data);
  },

  async getCombinedQuorum(
    id: string,
    query: CombinedQuorumQuery,
  ): Promise<CombinedQuorumResponse> {
    // #661 — l'URL était `/combined-quorum`, la route backend est `/quorum`
    // (`routes.rs:648`). Cet appel n'a jamais pu aboutir ; corrigé au passage.
    const params = new URLSearchParams({
      physical_quotas: String(query.physical_quotas),
      total_building_quotas: String(query.total_building_quotas),
      physical_owners_count: String(query.physical_owners_count),
      total_owners_count: String(query.total_owners_count),
    });
    return api.get(`/ag-sessions/${id}/quorum?${params}`);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/ag-sessions/${id}`);
  },
};
