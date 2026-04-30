import { api } from "../api";

/**
 * Marketplace API Client
 * Wraps backend endpoints for service provider directory (BC14)
 * Public search endpoint + authenticated management
 */

export type TradeCategory =
  | "Plumber"
  | "Electrician"
  | "Carpenter"
  | "Painter"
  | "Cleaner"
  | "Gardener"
  | "Locksmith"
  | "Roofer"
  | "HvacTechnician"
  | "GeneralContractor"
  | "Other";

export type ServiceProviderStatus = "Active" | "Inactive" | "Suspended";

export interface ServiceProvider {
  id: string;
  organization_id: string;
  company_name: string;
  slug: string;
  trade_category: TradeCategory;
  bce_number?: string;
  contact_email?: string;
  phone?: string;
  description?: string;
  postal_code?: string;
  city?: string;
  status: ServiceProviderStatus;
  average_rating?: number;
  total_evaluations: number;
  created_at: string;
  updated_at: string;
}

export interface CreateServiceProviderDto {
  company_name: string;
  trade_category: TradeCategory;
  bce_number?: string;
  contact_email?: string;
  phone?: string;
  description?: string;
  postal_code?: string;
  city?: string;
}

export const marketplaceApi = {
  async searchProviders(params?: {
    trade_category?: string;
    city?: string;
    q?: string;
  }): Promise<ServiceProvider[]> {
    const query = params
      ? "?" + new URLSearchParams(params as Record<string, string>).toString()
      : "";
    return api.get(`/marketplace/providers${query}`);
  },

  async getProviderBySlug(slug: string): Promise<ServiceProvider> {
    return api.get(`/marketplace/providers/${slug}`);
  },

  async create(data: CreateServiceProviderDto): Promise<ServiceProvider> {
    return api.post("/service-providers", data);
  },

  async getById(id: string): Promise<ServiceProvider> {
    return api.get(`/service-providers/${id}`);
  },
};
