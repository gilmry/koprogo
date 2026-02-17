import { api } from "../api";

/**
 * États Datés API Client
 * Belgian legal requirement: financial snapshot of a unit for property sales
 */

export enum EtatDateStatus {
  Requested = "requested",
  InProgress = "in_progress",
  Generated = "generated",
  Delivered = "delivered",
  Expired = "expired",
}

export enum EtatDateLanguage {
  Fr = "fr",
  Nl = "nl",
  De = "de",
}

export interface EtatDate {
  id: string;
  organization_id: string;
  building_id: string;
  unit_id: string;
  reference_date: string;
  requested_date: string;
  generated_date: string | null;
  delivered_date: string | null;
  status: EtatDateStatus;
  language: EtatDateLanguage;
  reference_number: string;
  notary_name: string;
  notary_email: string;
  notary_phone: string | null;
  building_name: string;
  building_address: string;
  unit_number: string;
  unit_floor: string | null;
  unit_area: number | null;
  ordinary_charges_quota: number;
  extraordinary_charges_quota: number;
  owner_balance: number;
  arrears_amount: number;
  monthly_provision_amount: number;
  total_balance: number;
  approved_works_unpaid: number;
  additional_data: Record<string, any>;
  pdf_file_path: string | null;
  created_at: string;
  updated_at: string;
  is_overdue: boolean;
  is_expired: boolean;
  days_since_request: number;
}

export interface CreateEtatDateDto {
  building_id: string;
  unit_id: string;
  reference_date: string;
  language: EtatDateLanguage;
  notary_name: string;
  notary_email: string;
  notary_phone?: string;
}

export interface UpdateFinancialDto {
  owner_balance: number;
  arrears_amount: number;
  monthly_provision_amount: number;
  total_balance: number;
  approved_works_unpaid: number;
}

export interface EtatDateStats {
  total_requests: number;
  requested_count: number;
  in_progress_count: number;
  generated_count: number;
  delivered_count: number;
  expired_count: number;
  overdue_count: number;
  average_processing_days: number;
}

export interface PageResponse<T> {
  data: T[];
  page: number;
  per_page: number;
  total: number;
}

export const etatsDatesApi = {
  async create(data: CreateEtatDateDto): Promise<EtatDate> {
    return api.post("/etats-dates", data);
  },

  async getById(id: string): Promise<EtatDate> {
    return api.get(`/etats-dates/${id}`);
  },

  async getByReference(referenceNumber: string): Promise<EtatDate> {
    return api.get(`/etats-dates/reference/${referenceNumber}`);
  },

  async list(
    page: number = 1,
    perPage: number = 10,
    status?: EtatDateStatus,
  ): Promise<PageResponse<EtatDate>> {
    let url = `/etats-dates?page=${page}&per_page=${perPage}`;
    if (status) url += `&status=${status}`;
    return api.get(url);
  },

  async listByUnit(unitId: string): Promise<EtatDate[]> {
    return api.get(`/units/${unitId}/etats-dates`);
  },

  async listByBuilding(buildingId: string): Promise<EtatDate[]> {
    return api.get(`/buildings/${buildingId}/etats-dates`);
  },

  async markInProgress(id: string): Promise<EtatDate> {
    return api.put(`/etats-dates/${id}/mark-in-progress`, {});
  },

  async markGenerated(id: string, pdfFilePath: string): Promise<EtatDate> {
    return api.put(`/etats-dates/${id}/mark-generated`, {
      pdf_file_path: pdfFilePath,
    });
  },

  async markDelivered(id: string): Promise<EtatDate> {
    return api.put(`/etats-dates/${id}/mark-delivered`, {});
  },

  async updateFinancial(
    id: string,
    data: UpdateFinancialDto,
  ): Promise<EtatDate> {
    return api.put(`/etats-dates/${id}/financial`, data);
  },

  async updateAdditionalData(
    id: string,
    additionalData: Record<string, any>,
  ): Promise<EtatDate> {
    return api.put(`/etats-dates/${id}/additional-data`, {
      additional_data: additionalData,
    });
  },

  async listOverdue(): Promise<EtatDate[]> {
    return api.get("/etats-dates/overdue");
  },

  async listExpired(): Promise<EtatDate[]> {
    return api.get("/etats-dates/expired");
  },

  async getStats(): Promise<EtatDateStats> {
    return api.get("/etats-dates/stats");
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/etats-dates/${id}`);
  },
};
