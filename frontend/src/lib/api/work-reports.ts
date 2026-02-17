import { api } from "../api";

/**
 * Work Report API Client
 * Wraps all backend endpoints for work report management (Digital Maintenance Logbook)
 */

export enum WorkType {
  Maintenance = "maintenance",
  Repair = "repair",
  Renovation = "renovation",
  Emergency = "emergency",
  Inspection = "inspection",
  Installation = "installation",
  Other = "other",
}

export enum WarrantyType {
  None = "none",
  Standard = "standard", // 2 years (vices apparents)
  Decennial = "decennial", // 10 years (garantie décennale)
  Extended = "extended", // 3 years
  Custom = "custom", // Custom duration
}

export interface CreateWorkReportDto {
  organization_id: string;
  building_id: string;
  title: string;
  description: string;
  work_type: WorkType;
  contractor_name: string;
  contractor_contact?: string;
  work_date: string;
  completion_date?: string;
  cost: number;
  invoice_number?: string;
  notes?: string;
  warranty_type: WarrantyType;
}

export interface UpdateWorkReportDto {
  title?: string;
  description?: string;
  work_type?: WorkType;
  contractor_name?: string;
  contractor_contact?: string;
  work_date?: string;
  completion_date?: string;
  cost?: number;
  invoice_number?: string;
  notes?: string;
  warranty_type?: WarrantyType;
}

export interface WorkReport {
  id: string;
  organization_id: string;
  building_id: string;
  title: string;
  description: string;
  work_type: WorkType;
  contractor_name: string;
  contractor_contact?: string;
  work_date: string;
  completion_date?: string;
  cost: number;
  invoice_number?: string;
  photos: string[];
  documents: string[];
  notes?: string;
  warranty_type: WarrantyType;
  warranty_expiry: string;
  is_warranty_valid: boolean;
  warranty_days_remaining: number;
  created_at: string;
  updated_at: string;
}

export interface WorkReportListResponse {
  work_reports: WorkReport[];
  total: number;
  page: number;
  page_size: number;
}

export interface WarrantyStatus {
  work_report_id: string;
  title: string;
  warranty_type: WarrantyType;
  warranty_expiry: string;
  is_valid: boolean;
  days_remaining: number;
}

export const workReportsApi = {
  async create(data: CreateWorkReportDto): Promise<WorkReport> {
    return api.post("/work-reports", data);
  },

  async getById(id: string): Promise<WorkReport> {
    return api.get(`/work-reports/${id}`);
  },

  async listByBuilding(buildingId: string): Promise<WorkReport[]> {
    return api.get(`/buildings/${buildingId}/work-reports`);
  },

  async listByOrganization(organizationId: string): Promise<WorkReport[]> {
    return api.get(`/organizations/${organizationId}/work-reports`);
  },

  async listPaginated(
    page: number = 1,
    pageSize: number = 20,
    buildingId?: string,
    workType?: WorkType,
  ): Promise<WorkReportListResponse> {
    const params = new URLSearchParams({
      page: String(page),
      page_size: String(pageSize),
    });
    if (buildingId) params.set("building_id", buildingId);
    if (workType) params.set("work_type", workType);
    return api.get(`/work-reports?${params.toString()}`);
  },

  async update(id: string, data: UpdateWorkReportDto): Promise<WorkReport> {
    return api.put(`/work-reports/${id}`, data);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/work-reports/${id}`);
  },

  async getActiveWarranties(buildingId: string): Promise<WarrantyStatus[]> {
    return api.get(`/buildings/${buildingId}/work-reports/warranties/active`);
  },

  async getExpiringWarranties(
    buildingId: string,
    days: number = 90,
  ): Promise<WarrantyStatus[]> {
    return api.get(
      `/buildings/${buildingId}/work-reports/warranties/expiring?days=${days}`,
    );
  },

  async addPhoto(id: string, photoPath: string): Promise<WorkReport> {
    return api.post(`/work-reports/${id}/photos`, { photo_path: photoPath });
  },

  async addDocument(id: string, documentPath: string): Promise<WorkReport> {
    return api.post(`/work-reports/${id}/documents`, {
      document_path: documentPath,
    });
  },
};

export const workTypeLabels: Record<WorkType, string> = {
  [WorkType.Maintenance]: "Entretien",
  [WorkType.Repair]: "Réparation",
  [WorkType.Renovation]: "Rénovation",
  [WorkType.Emergency]: "Urgence",
  [WorkType.Inspection]: "Inspection",
  [WorkType.Installation]: "Installation",
  [WorkType.Other]: "Autre",
};

export const warrantyTypeLabels: Record<WarrantyType, string> = {
  [WarrantyType.None]: "Aucune",
  [WarrantyType.Standard]: "Standard (2 ans)",
  [WarrantyType.Decennial]: "Décennale (10 ans)",
  [WarrantyType.Extended]: "Étendue (3 ans)",
  [WarrantyType.Custom]: "Personnalisée",
};
