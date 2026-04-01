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
  threshold?: number;
  // Backend field names (snake_case)
  vote_count_pour: number;
  vote_count_contre: number;
  vote_count_abstention: number;
  total_voting_power_pour: number;
  total_voting_power_contre: number;
  total_voting_power_abstention: number;
  // Computed by backend
  total_votes: number;
  pour_percentage: number;
  contre_percentage: number;
  abstention_percentage: number;
  // Aliases for backward compat (deprecated)
  votes_pour?: number;
  votes_contre?: number;
  votes_abstention?: number;
  total_voting_power?: number;
  status: ResolutionStatus;
  voted_at?: string;
  created_at: string;
  updated_at?: string;
}

export enum MajorityType {
  Simple = "simple",
  Absolute = "absolute",
  Qualified = "qualified",
}

export enum ResolutionStatus {
  Pending = "pending",
  Adopted = "adopted",
  Rejected = "rejected",
}

export enum VoteChoice {
  Pour = "pour",
  Contre = "contre",
  Abstention = "abstention",
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
  owner_id?: string;
  unit_id?: string;
  choice: VoteChoice;
  voting_power: number;
  proxy_owner_id?: string;
}

export const resolutionsApi = {
  async create(
    meetingId: string,
    data: CreateResolutionDto,
  ): Promise<Resolution> {
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
    // Mapper le champ 'choice' frontend vers 'vote_choice' backend
    const payload: Record<string, any> = {
      vote_choice: data.choice,
      voting_power: data.voting_power,
    };
    if (data.owner_id) payload.owner_id = data.owner_id;
    if (data.unit_id) payload.unit_id = data.unit_id;
    if (data.proxy_owner_id) payload.proxy_owner_id = data.proxy_owner_id;
    return api.post(`/resolutions/${resolutionId}/vote`, payload);
  },

  async getVotes(resolutionId: string): Promise<Vote[]> {
    return api.get(`/resolutions/${resolutionId}/votes`);
  },

  async changeVote(voteId: string, newChoice: VoteChoice): Promise<Vote> {
    return api.put(`/votes/${voteId}`, { vote_choice: newChoice });
  },

  async closeVoting(resolutionId: string): Promise<Resolution> {
    return api.put(`/resolutions/${resolutionId}/close`, {});
  },

  async getVoteSummary(meetingId: string): Promise<any> {
    return api.get(`/meetings/${meetingId}/vote-summary`);
  },
};
