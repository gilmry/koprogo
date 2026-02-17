import { api } from "../api";

/**
 * Technical Inspection API Client
 * Wraps all backend endpoints for technical inspection management
 */

export enum InspectionType {
  Elevator = "elevator",
  Boiler = "boiler",
  Electrical = "electrical",
  FireExtinguisher = "fire_extinguisher",
  FireAlarm = "fire_alarm",
  GasInstallation = "gas_installation",
  RoofStructure = "roof",
  Facade = "facade",
  WaterQuality = "water_tank",
  Drainage = "drainage",
  EmergencyLighting = "emergency_lighting",
  Other = "other",
}

export enum InspectionStatus {
  Pending = "pending",
  Completed = "completed",
  Failed = "failed",
  PassedWithRemarks = "passed_with_remarks",
}

export interface CreateInspectionDto {
  organization_id: string;
  building_id: string;
  title: string;
  description?: string;
  inspection_type: InspectionType;
  inspector_name: string;
  inspector_company?: string;
  inspector_certification?: string;
  inspection_date: string;
  result_summary?: string;
  defects_found?: string;
  recommendations?: string;
  compliant?: boolean;
  compliance_certificate_number?: string;
  compliance_valid_until?: string;
  cost?: number;
  invoice_number?: string;
  notes?: string;
}

export interface UpdateInspectionDto {
  title?: string;
  description?: string;
  inspection_type?: InspectionType;
  inspector_name?: string;
  inspector_company?: string;
  inspector_certification?: string;
  inspection_date?: string;
  status?: InspectionStatus;
  result_summary?: string;
  defects_found?: string;
  recommendations?: string;
  compliant?: boolean;
  compliance_certificate_number?: string;
  compliance_valid_until?: string;
  cost?: number;
  invoice_number?: string;
  notes?: string;
}

export interface TechnicalInspection {
  id: string;
  organization_id: string;
  building_id: string;
  title: string;
  description?: string;
  inspection_type: InspectionType;
  inspector_name: string;
  inspector_company?: string;
  inspector_certification?: string;
  inspection_date: string;
  next_due_date: string;
  status: InspectionStatus;
  result_summary?: string;
  defects_found?: string;
  recommendations?: string;
  compliant?: boolean;
  compliance_certificate_number?: string;
  compliance_valid_until?: string;
  cost?: number;
  invoice_number?: string;
  reports: string[];
  photos: string[];
  certificates: string[];
  notes?: string;
  is_overdue: boolean;
  days_until_due: number;
  created_at: string;
  updated_at: string;
}

export interface InspectionListResponse {
  inspections: TechnicalInspection[];
  total: number;
  page: number;
  page_size: number;
}

export const inspectionsApi = {
  async create(data: CreateInspectionDto): Promise<TechnicalInspection> {
    return api.post("/technical-inspections", data);
  },

  async getById(id: string): Promise<TechnicalInspection> {
    return api.get(`/technical-inspections/${id}`);
  },

  async listByBuilding(buildingId: string): Promise<TechnicalInspection[]> {
    return api.get(`/buildings/${buildingId}/technical-inspections`);
  },

  async listByOrganization(organizationId: string): Promise<TechnicalInspection[]> {
    return api.get(`/organizations/${organizationId}/technical-inspections`);
  },

  async listPaginated(page: number = 1, pageSize: number = 20, buildingId?: string, inspectionType?: string): Promise<InspectionListResponse> {
    const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) });
    if (buildingId) params.set("building_id", buildingId);
    if (inspectionType) params.set("inspection_type", inspectionType);
    return api.get(`/technical-inspections?${params.toString()}`);
  },

  async update(id: string, data: UpdateInspectionDto): Promise<TechnicalInspection> {
    return api.put(`/technical-inspections/${id}`, data);
  },

  async delete(id: string): Promise<void> {
    return api.delete(`/technical-inspections/${id}`);
  },

  async getOverdue(buildingId: string): Promise<TechnicalInspection[]> {
    return api.get(`/buildings/${buildingId}/technical-inspections/overdue`);
  },

  async getUpcoming(buildingId: string, days: number = 90): Promise<TechnicalInspection[]> {
    return api.get(`/buildings/${buildingId}/technical-inspections/upcoming?days=${days}`);
  },

  async getByType(buildingId: string, type: string): Promise<TechnicalInspection[]> {
    return api.get(`/buildings/${buildingId}/technical-inspections/type/${type}`);
  },

  async addReport(id: string, reportPath: string): Promise<TechnicalInspection> {
    return api.post(`/technical-inspections/${id}/reports`, { report_path: reportPath });
  },

  async addPhoto(id: string, photoPath: string): Promise<TechnicalInspection> {
    return api.post(`/technical-inspections/${id}/photos`, { photo_path: photoPath });
  },

  async addCertificate(id: string, certificatePath: string): Promise<TechnicalInspection> {
    return api.post(`/technical-inspections/${id}/certificates`, { certificate_path: certificatePath });
  },
};

export const inspectionTypeLabels: Record<InspectionType, string> = {
  [InspectionType.Elevator]: "Ascenseur",
  [InspectionType.Boiler]: "Chaudière",
  [InspectionType.Electrical]: "Installation électrique",
  [InspectionType.FireExtinguisher]: "Extincteurs",
  [InspectionType.FireAlarm]: "Alarme incendie",
  [InspectionType.GasInstallation]: "Installation gaz",
  [InspectionType.RoofStructure]: "Toiture",
  [InspectionType.Facade]: "Façade",
  [InspectionType.WaterQuality]: "Qualité eau",
  [InspectionType.Drainage]: "Drainage",
  [InspectionType.EmergencyLighting]: "Éclairage de secours",
  [InspectionType.Other]: "Autre",
};

export const inspectionStatusLabels: Record<InspectionStatus, string> = {
  [InspectionStatus.Pending]: "En attente",
  [InspectionStatus.Completed]: "Terminée",
  [InspectionStatus.Failed]: "Non conforme",
  [InspectionStatus.PassedWithRemarks]: "Conforme avec remarques",
};

export const inspectionFrequencyLabels: Record<InspectionType, string> = {
  [InspectionType.Elevator]: "Annuel",
  [InspectionType.Boiler]: "Annuel",
  [InspectionType.Electrical]: "5 ans",
  [InspectionType.FireExtinguisher]: "Annuel",
  [InspectionType.FireAlarm]: "Annuel",
  [InspectionType.GasInstallation]: "Annuel",
  [InspectionType.RoofStructure]: "5 ans",
  [InspectionType.Facade]: "10 ans",
  [InspectionType.WaterQuality]: "Annuel",
  [InspectionType.Drainage]: "5 ans",
  [InspectionType.EmergencyLighting]: "Annuel",
  [InspectionType.Other]: "Annuel",
};
