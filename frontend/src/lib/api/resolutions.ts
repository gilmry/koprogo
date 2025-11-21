import { api } from "../api";

/**
 * Resolution & Voting API Client
 * Belgian copropriété law compliance (tantièmes/millièmes voting)
 */

export interface Resolution {
  id: string;
  meeting_id: string;
  title: string;
  description: string;
  resolution_type: string;
  majority_required: MajorityType;
  threshold: number;
  votes_pour: number;
  votes_contre: number;
  votes_abstention: number;
  total_voting_power: number;
  status: ResolutionStatus;
  closed_at?: string;
  created_at: string;
  updated_at: string;
}

export enum MajorityType {
  Simple = "Simple",
  Absolute = "Absolute",
  Qualified = "Qualified",
}

export enum ResolutionStatus {
  Pending = "Pending",
  Adopted = "Adopted",
  Rejected = "Rejected",
}

export enum VoteChoice {
  Pour = "Pour",
  Contre = "Contre",
  Abstention = "Abstention",
}

export interface Vote {
  id: string;
  resolution_id: string;
  owner_id: string;
  owner_name?: string;
  choice: VoteChoice;
  voting_power: number;
  proxy_owner_id?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateResolutionDto {
  meeting_id: string;
  title: string;
  description: string;
  resolution_type: string;
  majority_required: MajorityType;
  threshold?: number;
}

export interface CastVoteDto {
  owner_id: string;
  choice: VoteChoice;
  voting_power: number;
  proxy_owner_id?: string;
}

export const resolutionsApi = {
  async create(meetingId: string, data: CreateResolutionDto): Promise<Resolution> {
    return api.post(`/meetings/${meetingId}/resolutions`, data);
  },

  async getById(id: string): Promise<Resolution> {
    return api.get(`/resolutions/${id}`);
  },

  async listByMeeting(meetingId: string): Promise<Resolution[]> {
    return api.get(`/meetings/${meetingId}/resolutions`);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/resolutions/${id}`);
  },

  async castVote(resolutionId: string, data: CastVoteDto): Promise<Vote> {
    return api.post(`/resolutions/${resolutionId}/vote`, data);
  },

  async getVotes(resolutionId: string): Promise<Vote[]> {
    return api.get(`/resolutions/${resolutionId}/votes`);
  },

  async changeVote(voteId: string, newChoice: VoteChoice): Promise<Vote> {
    return api.put(`/votes/${voteId}`, { choice: newChoice });
  },

  async closeVoting(resolutionId: string): Promise<Resolution> {
    return api.put(`/resolutions/${resolutionId}/close`, {});
  },

  async getVoteSummary(meetingId: string): Promise<any> {
    return api.get(`/meetings/${meetingId}/vote-summary`);
  },
};
