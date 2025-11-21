import { api } from "../api";

/**
 * Payment API Client
 * Wraps all 38 backend endpoints (22 payments + 16 payment methods)
 */

export interface Payment {
  id: string;
  organization_id: string;
  owner_id: string;
  expense_id?: string;
  building_id?: string;
  amount_cents: number;
  currency: string;
  status: PaymentStatus;
  payment_method_type: PaymentMethodType;
  stripe_payment_intent_id?: string;
  idempotency_key: string;
  refunded_amount_cents: number;
  refund_reason?: string;
  failure_reason?: string;
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export enum PaymentStatus {
  Pending = "Pending",
  Processing = "Processing",
  RequiresAction = "RequiresAction",
  Succeeded = "Succeeded",
  Failed = "Failed",
  Cancelled = "Cancelled",
  Refunded = "Refunded",
}

export enum PaymentMethodType {
  Card = "Card",
  SepaDebit = "SepaDebit",
  BankTransfer = "BankTransfer",
  Cash = "Cash",
}

export interface PaymentMethod {
  id: string;
  owner_id: string;
  organization_id: string;
  method_type: PaymentMethodType;
  stripe_payment_method_id?: string;
  display_label: string;
  last4?: string;
  brand?: string;
  expires_at?: string;
  is_default: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreatePaymentDto {
  owner_id: string;
  expense_id?: string;
  building_id?: string;
  amount_cents: number;
  currency?: string;
  payment_method_type: PaymentMethodType;
  stripe_payment_intent_id?: string;
  metadata?: Record<string, any>;
}

export interface CreatePaymentMethodDto {
  owner_id: string;
  method_type: PaymentMethodType;
  stripe_payment_method_id?: string;
  display_label: string;
  last4?: string;
  brand?: string;
  expires_at?: string;
}

export interface PaymentStats {
  total_paid_cents: number;
  succeeded_count: number;
  failed_count: number;
  refunded_count: number;
  net_amount_cents: number;
}

/**
 * Payments API functions (22 endpoints)
 */
export const paymentsApi = {
  /**
   * Create a new payment
   */
  async create(data: CreatePaymentDto): Promise<Payment> {
    return api.post("/payments", data);
  },

  /**
   * Get payment by ID
   */
  async getById(id: string): Promise<Payment> {
    return api.get(`/payments/${id}`);
  },

  /**
   * Get payment by Stripe Intent ID
   */
  async getByStripeIntentId(stripeIntentId: string): Promise<Payment> {
    return api.get(`/payments/stripe/${stripeIntentId}`);
  },

  /**
   * List payments by owner
   */
  async listByOwner(ownerId: string): Promise<Payment[]> {
    return api.get(`/owners/${ownerId}/payments`);
  },

  /**
   * List payments by building
   */
  async listByBuilding(buildingId: string): Promise<Payment[]> {
    return api.get(`/buildings/${buildingId}/payments`);
  },

  /**
   * List payments by expense
   */
  async listByExpense(expenseId: string): Promise<Payment[]> {
    return api.get(`/expenses/${expenseId}/payments`);
  },

  /**
   * List payments by organization
   */
  async listByOrganization(organizationId: string): Promise<Payment[]> {
    return api.get(`/organizations/${organizationId}/payments`);
  },

  /**
   * List payments by status
   */
  async listByStatus(status: PaymentStatus): Promise<Payment[]> {
    return api.get(`/payments/status/${status}`);
  },

  /**
   * Get pending payments
   */
  async getPending(): Promise<Payment[]> {
    return api.get("/payments/pending");
  },

  /**
   * Get failed payments
   */
  async getFailed(): Promise<Payment[]> {
    return api.get("/payments/failed");
  },

  /**
   * Mark payment as processing
   */
  async markAsProcessing(id: string): Promise<Payment> {
    return api.put(`/payments/${id}/processing`, {});
  },

  /**
   * Mark payment as requires action
   */
  async markAsRequiresAction(id: string): Promise<Payment> {
    return api.put(`/payments/${id}/requires-action`, {});
  },

  /**
   * Mark payment as succeeded
   */
  async markAsSucceeded(id: string): Promise<Payment> {
    return api.put(`/payments/${id}/succeeded`, {});
  },

  /**
   * Mark payment as failed
   */
  async markAsFailed(id: string, failureReason?: string): Promise<Payment> {
    return api.put(`/payments/${id}/failed`, { failure_reason: failureReason });
  },

  /**
   * Mark payment as cancelled
   */
  async markAsCancelled(id: string): Promise<Payment> {
    return api.put(`/payments/${id}/cancelled`, {});
  },

  /**
   * Process refund
   */
  async refund(
    id: string,
    amountCents?: number,
    reason?: string,
  ): Promise<Payment> {
    return api.post(`/payments/${id}/refund`, {
      amount_cents: amountCents,
      reason,
    });
  },

  /**
   * Delete payment
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/payments/${id}`);
  },

  /**
   * Get owner payment statistics
   */
  async getOwnerStats(ownerId: string): Promise<PaymentStats> {
    return api.get(`/owners/${ownerId}/payments/stats`);
  },

  /**
   * Get building payment statistics
   */
  async getBuildingStats(buildingId: string): Promise<PaymentStats> {
    return api.get(`/buildings/${buildingId}/payments/stats`);
  },

  /**
   * Get expense total paid
   */
  async getExpenseTotal(expenseId: string): Promise<{ total_paid_cents: number }> {
    return api.get(`/expenses/${expenseId}/payments/total`);
  },

  /**
   * Get owner total paid
   */
  async getOwnerTotal(ownerId: string): Promise<{ total_paid_cents: number }> {
    return api.get(`/owners/${ownerId}/payments/total`);
  },

  /**
   * Get building total paid
   */
  async getBuildingTotal(buildingId: string): Promise<{ total_paid_cents: number }> {
    return api.get(`/buildings/${buildingId}/payments/total`);
  },
};

/**
 * Payment Methods API functions (16 endpoints)
 */
export const paymentMethodsApi = {
  /**
   * Create a new payment method
   */
  async create(data: CreatePaymentMethodDto): Promise<PaymentMethod> {
    return api.post("/payment-methods", data);
  },

  /**
   * Get payment method by ID
   */
  async getById(id: string): Promise<PaymentMethod> {
    return api.get(`/payment-methods/${id}`);
  },

  /**
   * Get payment method by Stripe ID
   */
  async getByStripeId(stripeId: string): Promise<PaymentMethod> {
    return api.get(`/payment-methods/stripe/${stripeId}`);
  },

  /**
   * List payment methods by owner
   */
  async listByOwner(ownerId: string): Promise<PaymentMethod[]> {
    return api.get(`/owners/${ownerId}/payment-methods`);
  },

  /**
   * Get active payment methods by owner
   */
  async getActiveByOwner(ownerId: string): Promise<PaymentMethod[]> {
    return api.get(`/owners/${ownerId}/payment-methods/active`);
  },

  /**
   * Get default payment method by owner
   */
  async getDefaultByOwner(ownerId: string): Promise<PaymentMethod> {
    return api.get(`/owners/${ownerId}/payment-methods/default`);
  },

  /**
   * List payment methods by organization
   */
  async listByOrganization(organizationId: string): Promise<PaymentMethod[]> {
    return api.get(`/organizations/${organizationId}/payment-methods`);
  },

  /**
   * List payment methods by type
   */
  async listByType(
    ownerId: string,
    methodType: PaymentMethodType,
  ): Promise<PaymentMethod[]> {
    return api.get(`/owners/${ownerId}/payment-methods/type/${methodType}`);
  },

  /**
   * Update payment method
   */
  async update(
    id: string,
    data: Partial<PaymentMethod>,
  ): Promise<PaymentMethod> {
    return api.put(`/payment-methods/${id}`, data);
  },

  /**
   * Set as default payment method
   */
  async setAsDefault(id: string): Promise<void> {
    return api.put(`/payment-methods/${id}/set-default`, {});
  },

  /**
   * Deactivate payment method
   */
  async deactivate(id: string): Promise<void> {
    return api.put(`/payment-methods/${id}/deactivate`, {});
  },

  /**
   * Reactivate payment method
   */
  async reactivate(id: string): Promise<void> {
    return api.put(`/payment-methods/${id}/reactivate`, {});
  },

  /**
   * Delete payment method
   */
  async delete(id: string): Promise<void> {
    return api.delete(`/payment-methods/${id}`);
  },

  /**
   * Get count of active payment methods
   */
  async getCount(ownerId: string): Promise<number> {
    const response = await api.get<{ count: number }>(
      `/owners/${ownerId}/payment-methods/count`,
    );
    return response.count;
  },

  /**
   * Check if owner has active payment methods
   */
  async hasActive(ownerId: string): Promise<boolean> {
    const response = await api.get<{ has_active: boolean }>(
      `/owners/${ownerId}/payment-methods/has-active`,
    );
    return response.has_active;
  },
};
