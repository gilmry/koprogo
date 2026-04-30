import { api } from "../api";

/**
 * AG Sessions API Client
 * Wraps all 9 backend endpoints for AG video session management
 * Belgian legal compliance: Art. 3.87 §1 CC — remote participation in general assemblies
 */

export type VideoPlatform =
  | "Zoom"
  | "MicrosoftTeams"
  | "GoogleMeet"
  | "Jitsi"
  | "Whereby"
  | "Other";
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

export interface CombinedQuorumResponse {
  physical_quotas: number;
  remote_quotas: number;
  total_quotas: number;
  combined_percentage: number;
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

  async getCombinedQuorum(id: string): Promise<CombinedQuorumResponse> {
    return api.get(`/ag-sessions/${id}/combined-quorum`);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/ag-sessions/${id}`);
  },
};
