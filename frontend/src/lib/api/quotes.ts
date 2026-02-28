import { api } from "../api";

/**
 * Quote API Client
 * Wraps all 15 backend endpoints for contractor quote management
 * Belgian professional best practice: 3 quotes for works >5000€
 */

export interface Quote {
  id: string;
  building_id: string;
  contractor_id: string;
  contractor_name?: string;
  project_title: string;
  project_description: string;
  work_category: string;
  amount_excl_vat_cents?: number;
  vat_rate?: number;
  amount_incl_vat_cents?: number;
  validity_date?: string;
  estimated_duration_days?: number;
  warranty_years?: number;
  contractor_rating?: number;
  status: QuoteStatus;
  submitted_at?: string;
  decision_at?: string;
  decision_by?: string;
  decision_notes?: string;
  created_at: string;
  updated_at: string;
}

export enum QuoteStatus {
  Requested = "Requested",
  Received = "Received",
  UnderReview = "UnderReview",
  Accepted = "Accepted",
  Rejected = "Rejected",
  Expired = "Expired",
  Withdrawn = "Withdrawn",
}

export interface CreateQuoteDto {
  building_id: string;
  contractor_id: string;
  project_title: string;
  project_description: string;
  work_category: string;
}

export interface SubmitQuoteDto {
  amount_excl_vat_cents: number;
  vat_rate: number;
  validity_date: string;
  estimated_duration_days: number;
  warranty_years: number;
}

export interface AcceptQuoteDto {
  decision_by: string;
  decision_notes?: string;
}

export interface RejectQuoteDto {
  decision_by: string;
  decision_notes: string;
}

export interface QuoteComparison {
  quotes: QuoteWithScore[];
  recommendation: string;
  complies_with_belgian_law: boolean;
}

export interface QuoteWithScore {
  quote: Quote;
  score: number;
  price_score: number;
  delay_score: number;
  warranty_score: number;
  reputation_score: number;
}

/**
 * Quotes API functions
 */
export const quotesApi = {
  /**
   * Create a new quote request (Syndic initiates)
   */
  async create(data: CreateQuoteDto): Promise<Quote> {
    return api.post("/quotes", data);
  },

  /**
   * Get quote by ID
   */
  async getById(id: string): Promise<Quote> {
    return api.get(`/quotes/${id}`);
  },

  /**
   * List quotes by building
   */
  async listByBuilding(buildingId: string): Promise<Quote[]> {
    return api.get(`/buildings/${buildingId}/quotes`);
  },

  /**
   * List quotes by contractor
   */
  async listByContractor(contractorId: string): Promise<Quote[]> {
    return api.get(`/contractors/${contractorId}/quotes`);
  },

  /**
   * List quotes by status for building
   */
  async listByStatus(
    buildingId: string,
    status: QuoteStatus,
  ): Promise<Quote[]> {
    return api.get(`/buildings/${buildingId}/quotes/status/${status}`);
  },

  /**
   * Submit quote with pricing (Contractor action)
   */
  async submit(id: string, data: SubmitQuoteDto): Promise<Quote> {
    return api.post(`/quotes/${id}/submit`, data);
  },

  /**
   * Start review (Syndic marks as under review)
   */
  async startReview(id: string): Promise<Quote> {
    return api.post(`/quotes/${id}/review`, {});
  },

  /**
   * Accept quote
   */
  async accept(id: string, data: AcceptQuoteDto): Promise<Quote> {
    return api.post(`/quotes/${id}/accept`, data);
  },

  /**
   * Reject quote
   */
  async reject(id: string, data: RejectQuoteDto): Promise<Quote> {
    return api.post(`/quotes/${id}/reject`, data);
  },

  /**
   * Withdraw quote (Contractor withdraws)
   */
  async withdraw(id: string): Promise<Quote> {
    return api.post(`/quotes/${id}/withdraw`, {});
  },

  /**
   * Compare multiple quotes (Belgian 3-quote algorithm)
   * Price: 40%, Delay: 30%, Warranty: 20%, Reputation: 10%
   */
  async compare(quoteIds: string[]): Promise<QuoteComparison> {
    return api.post("/quotes/compare", { quote_ids: quoteIds });
  },

  /**
   * Update contractor rating (0-100)
   */
  async updateRating(id: string, rating: number): Promise<Quote> {
    return api.put(`/quotes/${id}/contractor-rating`, { rating });
  },

  /**
   * Delete quote
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/quotes/${id}`);
  },

  /**
   * Get count of quotes for building
   */
  async getCountByBuilding(buildingId: string): Promise<number> {
    const response = await api.get<{ count: number }>(
      `/buildings/${buildingId}/quotes/count`,
    );
    return response.count;
  },

  /**
   * Get count of quotes by status for building
   */
  async getCountByStatus(
    buildingId: string,
    status: QuoteStatus,
  ): Promise<number> {
    const response = await api.get<{ count: number }>(
      `/buildings/${buildingId}/quotes/status/${status}/count`,
    );
    return response.count;
  },
};
