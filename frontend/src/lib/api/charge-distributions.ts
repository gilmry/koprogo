import { api } from "../api";

/**
 * Charge Distribution API Client
 * Automatic expense distribution by ownership percentages (quotes-parts)
 */

export interface ChargeDistribution {
  id: string;
  expense_id: string;
  unit_id: string;
  owner_id: string;
  quota_percentage: number;
  amount_due: number;
  created_at: string;
}

export interface DistributionResult {
  message: string;
  count: number;
  distributions: ChargeDistribution[];
}

export interface OwnerTotalDue {
  owner_id: string;
  total_due: number;
}

export const chargeDistributionsApi = {
  async calculate(expenseId: string): Promise<DistributionResult> {
    return api.post(`/invoices/${expenseId}/calculate-distribution`, {});
  },

  async getByExpense(expenseId: string): Promise<ChargeDistribution[]> {
    return api.get(`/invoices/${expenseId}/distribution`);
  },

  async getByOwner(ownerId: string): Promise<ChargeDistribution[]> {
    return api.get(`/owners/${ownerId}/distributions`);
  },

  async getOwnerTotalDue(ownerId: string): Promise<OwnerTotalDue> {
    return api.get(`/owners/${ownerId}/total-due`);
  },
};
