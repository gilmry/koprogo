import { api } from "../api";

/**
 * AGE Requests API Client
 * Wraps all 11 backend endpoints for AGE (Extraordinary General Assembly) request management
 * Belgian legal compliance: Art. 3.87 §2 CC — owners can force extraordinary assembly
 * when representing at least 1/5 (20%) of total shares
 */

export type AgeRequestStatus =
  | "Draft"
  | "Open"
  | "Reached"
  | "Submitted"
  | "Accepted"
  | "Expired"
  | "Rejected"
  | "Withdrawn";

export interface AgeRequestCosignatory {
  owner_id: string;
  owner_name?: string;
  shares_pct: number;
  signed_at: string;
}

export interface AgeRequest {
  id: string;
  building_id: string;
  organization_id: string;
  initiated_by: string;
  title: string;
  description: string;
  status: AgeRequestStatus;
  threshold_pct: number;
  total_shares_pct: number;
  cosignatories: AgeRequestCosignatory[];
  submitted_to_syndic_at?: string;
  syndic_deadline_at?: string;
  auto_convocation_triggered: boolean;
  concertation_poll_id?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateAgeRequestDto {
  organization_id: string;
  title: string;
  description: string;
  threshold_pct?: number;
}

export interface CosignAgeRequestDto {
  owner_id: string;
  shares_pct: number;
}

export interface SyndicResponseDto {
  response: "Accept" | "Reject";
  reason?: string;
}

export const ageRequestsApi = {
  async create(
    buildingId: string,
    data: CreateAgeRequestDto,
  ): Promise<AgeRequest> {
    return api.post(`/buildings/${buildingId}/age-requests`, data);
  },

  async getById(id: string): Promise<AgeRequest> {
    return api.get(`/age-requests/${id}`);
  },

  async listByBuilding(buildingId: string): Promise<AgeRequest[]> {
    return api.get(`/buildings/${buildingId}/age-requests`);
  },

  async open(id: string): Promise<AgeRequest> {
    return api.put(`/age-requests/${id}/open`, {});
  },

  async cosign(id: string, data: CosignAgeRequestDto): Promise<AgeRequest> {
    return api.post(`/age-requests/${id}/cosign`, data);
  },

  async removeCosignatory(id: string, ownerId: string): Promise<AgeRequest> {
    return api.delete(`/age-requests/${id}/cosignatories/${ownerId}`);
  },

  async submit(id: string): Promise<AgeRequest> {
    return api.post(`/age-requests/${id}/submit`, {});
  },

  async accept(id: string): Promise<AgeRequest> {
    return api.post(`/age-requests/${id}/accept`, {});
  },

  async reject(id: string, reason?: string): Promise<AgeRequest> {
    return api.post(`/age-requests/${id}/reject`, { reason });
  },

  async withdraw(id: string): Promise<AgeRequest> {
    return api.post(`/age-requests/${id}/withdraw`, {});
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/age-requests/${id}`);
  },
};
