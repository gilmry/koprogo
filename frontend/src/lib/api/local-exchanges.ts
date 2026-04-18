import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Local Exchange Trading System (SEL) - Système d'Échange Local
 *
 * Time-based currency exchange system for co-owners to exchange services,
 * objects, and shared purchases using credits (1 hour = 1 credit).
 *
 * Belgian Legal Context:
 * - SELs are legal and recognized in Belgium
 * - No taxation if non-commercial (barter)
 * - Must not replace professional services (insurance issues)
 * - Clear T&Cs required (liability disclaimer)
 *
 * Enums are re-exported from auto-generated api.d.ts (STORY-P7-704) —
 * TypeScript will refuse any value that doesn't exist in the Rust enum.
 */

// ============================================================================
// Type Definitions
// ============================================================================

// Re-exported from generated api.d.ts — single source of truth.
export type ExchangeType = components["schemas"]["ExchangeType"];
export const ExchangeType = {
  Service: "Service" as const, // Skills (plumbing, gardening, tutoring, etc.)
  ObjectLoan: "ObjectLoan" as const, // Temporary loan (tools, books, equipment)
  SharedPurchase: "SharedPurchase" as const, // Co-buying (bulk food, equipment rental)
} satisfies Record<string, ExchangeType>;

export type ExchangeStatus = components["schemas"]["ExchangeStatus"];
export const ExchangeStatus = {
  Offered: "Offered" as const, // Available for anyone to request
  Requested: "Requested" as const, // Someone claimed it (pending provider acceptance)
  InProgress: "InProgress" as const, // Exchange is happening
  Completed: "Completed" as const, // Both parties confirmed completion
  Cancelled: "Cancelled" as const, // Exchange was cancelled
} satisfies Record<string, ExchangeStatus>;

export type CreditStatus = components["schemas"]["CreditStatus"];
export const CreditStatus = {
  Positive: "Positive" as const, // Balance > 0 (net provider)
  Balanced: "Balanced" as const, // Balance = 0
  Negative: "Negative" as const, // Balance < 0 (net receiver)
} satisfies Record<string, CreditStatus>;

export type ParticipationLevel = components["schemas"]["ParticipationLevel"];
export const ParticipationLevel = {
  New: "New" as const, // 0 exchanges
  Beginner: "Beginner" as const, // 1-5 exchanges
  Active: "Active" as const, // 6-20 exchanges
  Veteran: "Veteran" as const, // 21-50 exchanges
  Expert: "Expert" as const, // 51+ exchanges
} satisfies Record<string, ParticipationLevel>;

export interface LocalExchange {
  id: string;
  building_id: string;
  provider_id: string;
  provider_name: string; // Joined from owner table
  requester_id?: string;
  requester_name?: string; // Joined from owner table
  exchange_type: ExchangeType;
  title: string;
  description: string;
  credits: number; // Time in hours (1 hour = 1 credit)
  status: ExchangeStatus;
  offered_at: string;
  requested_at?: string;
  started_at?: string;
  completed_at?: string;
  cancelled_at?: string;
  cancellation_reason?: string;
  provider_rating?: number; // 1-5 stars from requester
  requester_rating?: number; // 1-5 stars from provider
  created_at: string;
  updated_at: string;
}

export interface OwnerCreditBalance {
  owner_id: string;
  owner_name: string; // Joined from owner table
  building_id: string;
  credits_earned: number;
  credits_spent: number;
  balance: number;
  credit_status: CreditStatus;
  total_exchanges: number;
  average_rating?: number; // 1-5 stars
  participation_level: ParticipationLevel;
  created_at: string;
  updated_at: string;
}

export interface SelStatistics {
  building_id: string;
  total_exchanges: number;
  active_exchanges: number;
  completed_exchanges: number;
  total_credits_exchanged: number;
  active_participants: number;
  average_exchange_rating?: number;
  most_popular_exchange_type?: ExchangeType;
}

export interface OwnerExchangeSummary {
  owner_id: string;
  owner_name: string;
  as_provider: number; // Number of exchanges as provider
  as_requester: number; // Number of exchanges as requester
  total_exchanges: number; // Sum of both
  average_rating?: number;
  recent_exchanges: LocalExchange[]; // Last 5
}

// ============================================================================
// DTOs
// ============================================================================

export interface CreateLocalExchangeDto {
  building_id: string;
  exchange_type: ExchangeType;
  title: string;
  description: string;
  credits: number; // Time in hours (1-100)
}

export interface CancelExchangeDto {
  reason?: string;
}

export interface RateExchangeDto {
  rating: number; // 1-5 stars
}

// ============================================================================
// API Client
// ============================================================================

export const localExchangesApi = {
  /**
   * Create a new exchange offer
   */
  async create(data: CreateLocalExchangeDto): Promise<LocalExchange> {
    return api.post("/exchanges", data);
  },

  /**
   * Get exchange by ID
   */
  async getById(id: string): Promise<LocalExchange> {
    return api.get(`/exchanges/${id}`);
  },

  /**
   * List all exchanges for a building
   */
  async listByBuilding(buildingId: string): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges`);
  },

  /**
   * List available exchanges (status = Offered)
   */
  async listAvailable(buildingId: string): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges/available`);
  },

  /**
   * List owner exchanges (as provider OR requester)
   */
  async listByOwner(ownerId: string): Promise<LocalExchange[]> {
    return api.get(`/owners/${ownerId}/exchanges`);
  },

  /**
   * List exchanges by type
   */
  async listByType(
    buildingId: string,
    exchangeType: ExchangeType,
  ): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges/type/${exchangeType}`);
  },

  /**
   * Request an exchange (Offered → Requested)
   */
  async request(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/request`, {});
  },

  /**
   * Start an exchange (Requested → InProgress)
   */
  async start(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/start`, {});
  },

  /**
   * Complete an exchange (InProgress → Completed)
   * Automatically updates credit balances for both parties
   */
  async complete(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/complete`, {});
  },

  /**
   * Cancel an exchange
   */
  async cancel(id: string, data: CancelExchangeDto): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/cancel`, data);
  },

  /**
   * Rate provider (requester rates provider after completion)
   */
  async rateProvider(id: string, data: RateExchangeDto): Promise<void> {
    return api.put(`/exchanges/${id}/rate-provider`, data);
  },

  /**
   * Rate requester (provider rates requester after completion)
   */
  async rateRequester(id: string, data: RateExchangeDto): Promise<void> {
    return api.put(`/exchanges/${id}/rate-requester`, data);
  },

  /**
   * Delete an exchange (provider only, not completed)
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/exchanges/${id}`);
  },

  /**
   * Get owner credit balance
   */
  async getCreditBalance(
    ownerId: string,
    buildingId: string,
  ): Promise<OwnerCreditBalance> {
    return api.get(`/owners/${ownerId}/buildings/${buildingId}/credit-balance`);
  },

  /**
   * Get building leaderboard (top contributors)
   */
  async getLeaderboard(
    buildingId: string,
    limit: number = 10,
  ): Promise<OwnerCreditBalance[]> {
    let url = `/buildings/${buildingId}/leaderboard`;
    if (limit) url += `?limit=${limit}`;
    return api.get(url);
  },

  /**
   * Get building SEL statistics
   */
  async getStatistics(buildingId: string): Promise<SelStatistics> {
    return api.get(`/buildings/${buildingId}/sel-statistics`);
  },

  /**
   * Get owner exchange summary
   */
  async getOwnerSummary(ownerId: string): Promise<OwnerExchangeSummary> {
    return api.get(`/owners/${ownerId}/exchange-summary`);
  },
};

/**
 * Helper functions
 */

export const exchangeTypeLabels: Record<ExchangeType, string> = {
  [ExchangeType.Service]: "Service",
  [ExchangeType.ObjectLoan]: "Prêt d'objet",
  [ExchangeType.SharedPurchase]: "Achat groupé",
};

export const exchangeTypeIcons: Record<ExchangeType, string> = {
  [ExchangeType.Service]: "🛠️",
  [ExchangeType.ObjectLoan]: "📦",
  [ExchangeType.SharedPurchase]: "🛒",
};

export const exchangeStatusLabels: Record<ExchangeStatus, string> = {
  [ExchangeStatus.Offered]: "Disponible",
  [ExchangeStatus.Requested]: "Demandé",
  [ExchangeStatus.InProgress]: "En cours",
  [ExchangeStatus.Completed]: "Terminé",
  [ExchangeStatus.Cancelled]: "Annulé",
};

export const exchangeStatusColors: Record<
  ExchangeStatus,
  { bg: string; text: string }
> = {
  [ExchangeStatus.Offered]: {
    bg: "bg-green-100",
    text: "text-green-800",
  },
  [ExchangeStatus.Requested]: {
    bg: "bg-blue-100",
    text: "text-blue-800",
  },
  [ExchangeStatus.InProgress]: {
    bg: "bg-yellow-100",
    text: "text-yellow-800",
  },
  [ExchangeStatus.Completed]: {
    bg: "bg-gray-100",
    text: "text-gray-800",
  },
  [ExchangeStatus.Cancelled]: {
    bg: "bg-red-100",
    text: "text-red-800",
  },
};

export const participationLevelLabels: Record<ParticipationLevel, string> = {
  [ParticipationLevel.New]: "Nouveau",
  [ParticipationLevel.Beginner]: "Débutant",
  [ParticipationLevel.Active]: "Actif",
  [ParticipationLevel.Veteran]: "Vétéran",
  [ParticipationLevel.Expert]: "Expert",
};

export const participationLevelColors: Record<
  ParticipationLevel,
  { bg: string; text: string }
> = {
  [ParticipationLevel.New]: {
    bg: "bg-gray-100",
    text: "text-gray-800",
  },
  [ParticipationLevel.Beginner]: {
    bg: "bg-blue-100",
    text: "text-blue-800",
  },
  [ParticipationLevel.Active]: {
    bg: "bg-green-100",
    text: "text-green-800",
  },
  [ParticipationLevel.Veteran]: {
    bg: "bg-purple-100",
    text: "text-purple-800",
  },
  [ParticipationLevel.Expert]: {
    bg: "bg-yellow-100",
    text: "text-yellow-800",
  },
};

export function formatCredits(credits: number): string {
  if (credits === 1) {
    return "1 heure";
  }
  return `${credits} heures`;
}

export function formatRating(rating?: number): string {
  if (!rating) return "Pas encore noté";
  const stars = "⭐".repeat(Math.floor(rating));
  const emptyStars = "☆".repeat(5 - Math.floor(rating));
  return `${stars}${emptyStars} (${rating.toFixed(1)})`;
}

export function getCreditStatusColor(status: CreditStatus): string {
  switch (status) {
    case CreditStatus.Positive:
      return "text-green-600";
    case CreditStatus.Balanced:
      return "text-gray-600";
    case CreditStatus.Negative:
      return "text-orange-600";
  }
}
