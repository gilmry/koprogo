import { api } from "../api";

/**
 * Payment Reminders API Client
 * Belgian payment recovery workflow: FirstReminder (J+15) → SecondReminder (J+30) → FormalNotice (J+60)
 */

export enum ReminderLevel {
  FirstReminder = "FirstReminder",
  SecondReminder = "SecondReminder",
  FormalNotice = "FormalNotice",
  LegalAction = "LegalAction",
}

export enum ReminderStatus {
  Pending = "Pending",
  Sent = "Sent",
  Opened = "Opened",
  Paid = "Paid",
  Escalated = "Escalated",
  Cancelled = "Cancelled",
}

export enum DeliveryMethod {
  Email = "Email",
  RegisteredLetter = "RegisteredLetter",
  Bailiff = "Bailiff",
}

export interface PaymentReminder {
  id: string;
  organization_id: string;
  expense_id: string;
  owner_id: string;
  owner_name?: string;
  owner_email?: string;
  level: ReminderLevel;
  status: ReminderStatus;
  amount_owed: number;
  penalty_amount: number;
  total_amount: number;
  due_date: string;
  days_overdue: number;
  delivery_method: DeliveryMethod;
  sent_date: string | null;
  opened_date: string | null;
  pdf_path: string | null;
  tracking_number: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateReminderDto {
  expense_id: string;
  owner_id: string;
  level: ReminderLevel;
  amount_owed: number;
  due_date: string;
  days_overdue: number;
}

export interface RecoveryStats {
  total_owed: number;
  total_penalties: number;
  reminder_counts: { level: string; count: number }[];
  status_counts: { status: string; count: number }[];
}

export interface BulkCreateResponse {
  created_count: number;
  skipped_count: number;
  errors: string[];
  created_reminders: PaymentReminder[];
}

export interface OverdueExpense {
  expense_id: string;
  owner_id: string;
  days_overdue: number;
  amount: number;
  recommended_level: ReminderLevel;
}

export const paymentRemindersApi = {
  async create(data: CreateReminderDto): Promise<PaymentReminder> {
    return api.post("/payment-reminders", data);
  },

  async getById(id: string): Promise<PaymentReminder> {
    return api.get(`/payment-reminders/${id}`);
  },

  async list(): Promise<PaymentReminder[]> {
    return api.get("/payment-reminders");
  },

  async listByExpense(expenseId: string): Promise<PaymentReminder[]> {
    return api.get(`/expenses/${expenseId}/payment-reminders`);
  },

  async listByOwner(ownerId: string): Promise<PaymentReminder[]> {
    return api.get(`/owners/${ownerId}/payment-reminders`);
  },

  async listActiveByOwner(ownerId: string): Promise<PaymentReminder[]> {
    return api.get(`/owners/${ownerId}/payment-reminders/active`);
  },

  async markAsSent(id: string, pdfPath?: string): Promise<PaymentReminder> {
    return api.put(`/payment-reminders/${id}/mark-sent`, {
      pdf_path: pdfPath || null,
    });
  },

  async markAsOpened(id: string): Promise<PaymentReminder> {
    return api.put(`/payment-reminders/${id}/mark-opened`, {});
  },

  async markAsPaid(id: string): Promise<PaymentReminder> {
    return api.put(`/payment-reminders/${id}/mark-paid`, {});
  },

  async cancel(id: string, reason: string): Promise<PaymentReminder> {
    return api.put(`/payment-reminders/${id}/cancel`, { reason });
  },

  async escalate(id: string, reason?: string): Promise<PaymentReminder> {
    return api.post(`/payment-reminders/${id}/escalate`, {
      reason: reason || null,
    });
  },

  async addTrackingNumber(
    id: string,
    trackingNumber: string,
  ): Promise<PaymentReminder> {
    return api.put(`/payment-reminders/${id}/tracking-number`, {
      tracking_number: trackingNumber,
    });
  },

  async getStats(): Promise<RecoveryStats> {
    return api.get("/payment-reminders/stats");
  },

  async findOverdueWithoutReminders(
    minDaysOverdue: number = 15,
  ): Promise<OverdueExpense[]> {
    return api.get(
      `/payment-reminders/overdue-without-reminders?min_days_overdue=${minDaysOverdue}`,
    );
  },

  async bulkCreate(minDaysOverdue: number = 15): Promise<BulkCreateResponse> {
    return api.post("/payment-reminders/bulk-create", {
      min_days_overdue: minDaysOverdue,
    });
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/payment-reminders/${id}`);
  },
};
