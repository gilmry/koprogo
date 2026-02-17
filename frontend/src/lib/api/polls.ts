import { api } from "../api";

/**
 * Polls API Client
 * Wraps all 12 backend endpoints for poll (sondage) management
 * Belgian legal compliance: Article 577-8/4 §4 Code Civil Belge
 * Allows syndic to consult co-owners between general assemblies
 */

export interface Poll {
  id: string;
  building_id: string;
  created_by: string;
  poll_type: PollType;
  title: string;
  description?: string;
  status: PollStatus;
  starts_at: string;
  ends_at: string;
  is_anonymous: boolean;
  total_eligible_voters: number;
  total_votes_cast: number;
  allow_multiple_votes: boolean;
  require_all_owners: boolean;
  participation_rate: number;
  is_active: boolean;
  is_ended: boolean;
  winning_option?: PollOption;
  options: PollOption[];
  created_at: string;
  updated_at: string;
}

export enum PollType {
  YesNo = "yes_no",
  MultipleChoice = "multiple_choice",
  Rating = "rating",
  OpenEnded = "open_ended",
}

export enum PollStatus {
  Draft = "draft",
  Active = "active",
  Closed = "closed",
  Cancelled = "cancelled",
}

export interface PollOption {
  id: string;
  option_text: string;
  attachment_url?: string;
  vote_count: number;
  vote_percentage: number;
  display_order: number;
}

export interface PollVote {
  id: string;
  poll_id: string;
  owner_id?: string;
  building_id: string;
  selected_option_ids: string[];
  rating_value?: number;
  open_text?: string;
  voted_at: string;
}

export interface PollResults {
  poll_id: string;
  total_votes_cast: number;
  total_eligible_voters: number;
  participation_rate: number;
  options: PollOption[];
  winning_option?: PollOption;
}

export interface PollStatistics {
  building_id: string;
  total_polls: number;
  active_polls: number;
  closed_polls: number;
  total_votes_cast: number;
  average_participation_rate: number;
}

export interface CreatePollDto {
  building_id: string;
  title: string;
  description?: string;
  poll_type: PollType | string;
  options: CreatePollOptionDto[];
  is_anonymous?: boolean;
  allow_multiple_votes?: boolean;
  require_all_owners?: boolean;
  ends_at: string;
}

export interface CreatePollOptionDto {
  option_text: string;
  attachment_url?: string;
  display_order: number;
}

export interface UpdatePollDto {
  title?: string;
  description?: string;
  options?: CreatePollOptionDto[];
  is_anonymous?: boolean;
  allow_multiple_votes?: boolean;
  require_all_owners?: boolean;
  ends_at?: string;
}

export interface CastVoteDto {
  poll_id: string;
  selected_option_ids?: string[]; // For YesNo/MultipleChoice
  rating_value?: number; // For Rating (1-5)
  open_text?: string; // For OpenEnded
}

export interface PublishPollDto {
  starts_at?: string;
  ends_at?: string;
}

/**
 * Polls API functions
 */
export const pollsApi = {
  /**
   * Create a new poll (draft status)
   */
  async create(data: CreatePollDto): Promise<Poll> {
    return api.post("/polls", data);
  },

  /**
   * Get poll by ID
   */
  async getById(id: string): Promise<Poll> {
    return api.get(`/polls/${id}`);
  },

  /**
   * Update poll (only draft polls can be updated)
   */
  async update(id: string, data: UpdatePollDto): Promise<Poll> {
    return api.put(`/polls/${id}`, data);
  },

  /**
   * List polls with filters
   */
  async list(filters?: {
    building_id?: string;
    status?: PollStatus;
    page?: number;
    page_size?: number;
  }): Promise<Poll[]> {
    let url = "/polls";
    if (filters) {
      const params = new URLSearchParams();
      if (filters.building_id)
        params.append("building_id", filters.building_id);
      if (filters.status) params.append("status", filters.status);
      if (filters.page) params.append("page", filters.page.toString());
      if (filters.page_size)
        params.append("page_size", filters.page_size.toString());
      if (params.toString()) url += `?${params.toString()}`;
    }
    const response = await api.get(url);
    return response.polls ?? response;
  },

  /**
   * List active polls for a building
   */
  async listActive(buildingId: string): Promise<Poll[]> {
    return api.get(`/buildings/${buildingId}/polls/active`);
  },

  /**
   * Publish poll (Draft → Active)
   */
  async publish(id: string, data: PublishPollDto): Promise<Poll> {
    return api.post(`/polls/${id}/publish`, data);
  },

  /**
   * Close poll (Active → Closed, calculate results)
   */
  async close(id: string): Promise<Poll> {
    return api.post(`/polls/${id}/close`, {});
  },

  /**
   * Cancel poll (→ Cancelled)
   */
  async cancel(id: string): Promise<Poll> {
    return api.post(`/polls/${id}/cancel`, {});
  },

  /**
   * Delete poll
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/polls/${id}`);
  },

  /**
   * Cast vote on a poll
   */
  async vote(data: CastVoteDto): Promise<PollVote> {
    return api.post("/polls/vote", data);
  },

  /**
   * Get poll results (vote counts, percentages, winner)
   */
  async getResults(id: string): Promise<PollResults> {
    return api.get(`/polls/${id}/results`);
  },

  /**
   * Get building poll statistics
   */
  async getStatistics(buildingId: string): Promise<PollStatistics> {
    return api.get(`/buildings/${buildingId}/polls/statistics`);
  },
};
