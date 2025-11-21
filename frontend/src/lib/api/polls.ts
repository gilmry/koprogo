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
  question: string;
  description?: string;
  status: PollStatus;
  starts_at?: string;
  ends_at?: string;
  is_anonymous: boolean;
  total_eligible_voters: number;
  total_votes_cast: number;
  allow_multiple_votes: boolean;
  min_rating?: number;
  max_rating?: number;
  options: PollOption[];
  created_at: string;
  updated_at: string;
}

export enum PollType {
  YesNo = "YesNo",
  MultipleChoice = "MultipleChoice",
  Rating = "Rating",
  OpenEnded = "OpenEnded",
}

export enum PollStatus {
  Draft = "Draft",
  Active = "Active",
  Closed = "Closed",
  Cancelled = "Cancelled",
}

export interface PollOption {
  id: string;
  poll_id: string;
  option_text: string;
  option_value?: number;
  display_order: number;
  vote_count: number;
  created_at: string;
}

export interface PollVote {
  id: string;
  poll_id: string;
  owner_id?: string; // NULL for anonymous votes
  option_id?: string; // For multiple choice
  vote_value?: number; // For rating
  vote_text?: string; // For open-ended
  ip_address?: string;
  is_anonymous: boolean;
  created_at: string;
}

export interface PollResults {
  poll_id: string;
  total_votes: number;
  participation_rate: number;
  results_by_option: OptionResult[];
  winner?: OptionResult; // For YesNo/MultipleChoice
  average_rating?: number; // For Rating
  text_responses?: string[]; // For OpenEnded
}

export interface OptionResult {
  option_id: string;
  option_text: string;
  vote_count: number;
  percentage: number;
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
  poll_type: PollType;
  question: string;
  description?: string;
  starts_at?: string;
  ends_at?: string;
  is_anonymous?: boolean;
  allow_multiple_votes?: boolean;
  min_rating?: number;
  max_rating?: number;
  options?: CreatePollOptionDto[];
}

export interface CreatePollOptionDto {
  option_text: string;
  option_value?: number;
  display_order: number;
}

export interface UpdatePollDto {
  question?: string;
  description?: string;
  starts_at?: string;
  ends_at?: string;
  is_anonymous?: boolean;
  allow_multiple_votes?: boolean;
}

export interface CastVoteDto {
  poll_id: string;
  option_id?: string; // For YesNo/MultipleChoice
  vote_value?: number; // For Rating
  vote_text?: string; // For OpenEnded
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
      if (filters.building_id) params.append("building_id", filters.building_id);
      if (filters.status) params.append("status", filters.status);
      if (filters.page) params.append("page", filters.page.toString());
      if (filters.page_size) params.append("page_size", filters.page_size.toString());
      if (params.toString()) url += `?${params.toString()}`;
    }
    return api.get(url);
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
