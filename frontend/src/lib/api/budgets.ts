import { api } from "../api";

/**
 * Budgets API Client
 * Belgian copropriété annual budget management with AG approval workflow
 */

export enum BudgetStatus {
  Draft = "draft",
  Submitted = "submitted",
  Approved = "approved",
  Rejected = "rejected",
  Archived = "archived",
}

export interface Budget {
  id: string;
  organization_id: string;
  building_id: string;
  fiscal_year: number;
  ordinary_budget: number;
  extraordinary_budget: number;
  total_budget: number;
  status: BudgetStatus;
  submitted_date: string | null;
  approved_date: string | null;
  approved_by_meeting_id: string | null;
  monthly_provision_amount: number;
  notes: string | null;
  created_at: string;
  updated_at: string;
  is_active: boolean;
  is_editable: boolean;
}

export interface CreateBudgetDto {
  building_id: string;
  fiscal_year: number;
  ordinary_budget: number;
  extraordinary_budget: number;
  notes?: string;
}

export interface UpdateBudgetDto {
  ordinary_budget?: number;
  extraordinary_budget?: number;
  notes?: string;
}

export interface BudgetStats {
  total_budgets: number;
  draft_count: number;
  submitted_count: number;
  approved_count: number;
  rejected_count: number;
  archived_count: number;
  average_total_budget: number;
  average_monthly_provision: number;
}

export interface BudgetVariance {
  budget_id: string;
  fiscal_year: number;
  building_id: string;
  budgeted_ordinary: number;
  budgeted_extraordinary: number;
  budgeted_total: number;
  actual_ordinary: number;
  actual_extraordinary: number;
  actual_total: number;
  variance_ordinary: number;
  variance_extraordinary: number;
  variance_total: number;
  variance_ordinary_pct: number;
  variance_extraordinary_pct: number;
  variance_total_pct: number;
  has_overruns: boolean;
  overrun_categories: string[];
  months_elapsed: number;
  projected_year_end_total: number;
}

export interface PageResponse<T> {
  data: T[];
  page: number;
  per_page: number;
  total: number;
}

export const budgetsApi = {
  async create(data: CreateBudgetDto): Promise<Budget> {
    return api.post("/budgets", data);
  },

  async getById(id: string): Promise<Budget> {
    return api.get(`/budgets/${id}`);
  },

  async update(id: string, data: UpdateBudgetDto): Promise<Budget> {
    return api.put(`/budgets/${id}`, data);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/budgets/${id}`);
  },

  async list(
    page: number = 1,
    perPage: number = 20,
    buildingId?: string,
    status?: BudgetStatus,
  ): Promise<PageResponse<Budget>> {
    let url = `/budgets?page=${page}&per_page=${perPage}`;
    if (buildingId) url += `&building_id=${buildingId}`;
    if (status) url += `&status=${status}`;
    return api.get(url);
  },

  async listByFiscalYear(fiscalYear: number): Promise<Budget[]> {
    return api.get(`/budgets/fiscal-year/${fiscalYear}`);
  },

  async listByStatus(status: BudgetStatus): Promise<Budget[]> {
    return api.get(`/budgets/status/${status}`);
  },

  async getStats(): Promise<BudgetStats> {
    return api.get("/budgets/stats");
  },

  async getVariance(id: string): Promise<BudgetVariance> {
    return api.get(`/budgets/${id}/variance`);
  },

  async listByBuilding(buildingId: string): Promise<Budget[]> {
    return api.get(`/buildings/${buildingId}/budgets`);
  },

  async getActiveByBuilding(buildingId: string): Promise<Budget> {
    return api.get(`/buildings/${buildingId}/budgets/active`);
  },

  async getByBuildingAndYear(
    buildingId: string,
    fiscalYear: number,
  ): Promise<Budget> {
    return api.get(
      `/buildings/${buildingId}/budgets/fiscal-year/${fiscalYear}`,
    );
  },

  async submit(id: string): Promise<Budget> {
    return api.put(`/budgets/${id}/submit`, {});
  },

  async approve(id: string, meetingId: string): Promise<Budget> {
    return api.put(`/budgets/${id}/approve`, { meeting_id: meetingId });
  },

  async reject(id: string, reason?: string): Promise<Budget> {
    return api.put(`/budgets/${id}/reject`, { reason: reason || null });
  },

  async archive(id: string): Promise<Budget> {
    return api.put(`/budgets/${id}/archive`, {});
  },
};
