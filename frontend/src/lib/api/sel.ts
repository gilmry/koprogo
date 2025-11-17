import { api } from "../api";

/**
 * SEL (Système d'Échange Local) API Client
 * Time-based currency: 1 hour = 1 credit
 * Belgian legal: non-taxable if non-commercial
 */

export interface LocalExchange {
  id: string;
  building_id: string;
  provider_id: string;
  provider_name?: string;
  requester_id?: string;
  requester_name?: string;
  exchange_type: ExchangeType;
  title: string;
  description: string;
  credits: number;
  status: ExchangeStatus;
  provider_rating?: number;
  requester_rating?: number;
  cancellation_reason?: string;
  created_at: string;
  updated_at: string;
}

export enum ExchangeType {
  Service = "Service",
  ObjectLoan = "ObjectLoan",
  SharedPurchase = "SharedPurchase",
}

export enum ExchangeStatus {
  Offered = "Offered",
  Requested = "Requested",
  InProgress = "InProgress",
  Completed = "Completed",
  Cancelled = "Cancelled",
}

export interface OwnerCreditBalance {
  owner_id: string;
  owner_name?: string;
  building_id: string;
  credits_earned: number;
  credits_spent: number;
  balance: number;
  total_exchanges: number;
  average_rating?: number;
  participation_level: string;
}

export interface CreateExchangeDto {
  building_id: string;
  provider_id: string;
  exchange_type: ExchangeType;
  title: string;
  description: string;
  credits: number;
}

export const selApi = {
  async create(data: CreateExchangeDto): Promise<LocalExchange> {
    return api.post("/exchanges", data);
  },

  async getById(id: string): Promise<LocalExchange> {
    return api.get(`/exchanges/${id}`);
  },

  async listByBuilding(buildingId: string): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges`);
  },

  async getAvailable(buildingId: string): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges/available`);
  },

  async listByOwner(ownerId: string): Promise<LocalExchange[]> {
    return api.get(`/owners/${ownerId}/exchanges`);
  },

  async listByType(buildingId: string, exchangeType: ExchangeType): Promise<LocalExchange[]> {
    return api.get(`/buildings/${buildingId}/exchanges/type/${exchangeType}`);
  },

  async request(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/request`, {});
  },

  async start(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/start`, {});
  },

  async complete(id: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/complete`, {});
  },

  async cancel(id: string, reason: string): Promise<LocalExchange> {
    return api.post(`/exchanges/${id}/cancel`, { reason });
  },

  async rateProvider(id: string, rating: number): Promise<LocalExchange> {
    return api.put(`/exchanges/${id}/rate-provider`, { rating });
  },

  async rateRequester(id: string, rating: number): Promise<LocalExchange> {
    return api.put(`/exchanges/${id}/rate-requester`, { rating });
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/exchanges/${id}`);
  },

  async getCreditBalance(ownerId: string, buildingId: string): Promise<OwnerCreditBalance> {
    return api.get(`/owners/${ownerId}/buildings/${buildingId}/credit-balance`);
  },

  async getLeaderboard(buildingId: string, limit = 10): Promise<OwnerCreditBalance[]> {
    return api.get(`/buildings/${buildingId}/leaderboard?limit=${limit}`);
  },

  async getStatistics(buildingId: string): Promise<any> {
    return api.get(`/buildings/${buildingId}/sel-statistics`);
  },

  async getOwnerSummary(ownerId: string): Promise<any> {
    return api.get(`/owners/${ownerId}/exchange-summary`);
  },
};
