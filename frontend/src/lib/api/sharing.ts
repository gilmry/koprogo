import { api } from "../api";

/**
 * Object Sharing Library API Client
 * Community tool/object lending system
 */

export interface SharedObject {
  id: string;
  building_id: string;
  owner_id: string;
  owner_name?: string;
  object_category: ObjectCategory;
  object_name: string;
  description: string;
  condition: ObjectCondition;
  availability_status: AvailabilityStatus;
  loan_duration_days: number;
  deposit_required_cents?: number;
  replacement_value_cents?: number;
  usage_instructions?: string;
  image_urls?: string[];
  total_loans: number;
  rating?: number;
  created_at: string;
  updated_at: string;
}

export enum ObjectCategory {
  Tools = "Tools",
  GardenEquipment = "GardenEquipment",
  KitchenAppliances = "KitchenAppliances",
  Electronics = "Electronics",
  Sports = "Sports",
  Books = "Books",
  ChildrenEquipment = "ChildrenEquipment",
  PartySupplies = "PartySupplies",
  Other = "Other",
}

export enum ObjectCondition {
  New = "New",
  LikeNew = "LikeNew",
  Good = "Good",
  Fair = "Fair",
  Poor = "Poor",
}

export enum AvailabilityStatus {
  Available = "Available",
  OnLoan = "OnLoan",
  Reserved = "Reserved",
  Unavailable = "Unavailable",
  Retired = "Retired",
}

export interface Loan {
  id: string;
  shared_object_id: string;
  object_name?: string;
  borrower_id: string;
  borrower_name?: string;
  lender_id: string;
  lender_name?: string;
  loan_start_date: string;
  loan_end_date: string;
  actual_return_date?: string;
  status: LoanStatus;
  deposit_paid_cents?: number;
  condition_at_loan: ObjectCondition;
  condition_at_return?: ObjectCondition;
  borrower_rating?: number;
  lender_rating?: number;
  notes?: string;
  created_at: string;
  updated_at: string;
}

export enum LoanStatus {
  Requested = "Requested",
  Approved = "Approved",
  Active = "Active",
  Returned = "Returned",
  Overdue = "Overdue",
  Cancelled = "Cancelled",
  Disputed = "Disputed",
}

export interface CreateSharedObjectDto {
  building_id: string;
  owner_id: string;
  object_category: ObjectCategory;
  object_name: string;
  description: string;
  condition: ObjectCondition;
  loan_duration_days: number;
  deposit_required_cents?: number;
  replacement_value_cents?: number;
  usage_instructions?: string;
}

export interface CreateLoanDto {
  shared_object_id: string;
  borrower_id: string;
  loan_start_date: string;
  loan_end_date: string;
  notes?: string;
}

export const sharingApi = {
  // Shared Objects
  async createObject(data: CreateSharedObjectDto): Promise<SharedObject> {
    return api.post("/shared-objects", data);
  },

  async getObjectById(id: string): Promise<SharedObject> {
    return api.get(`/shared-objects/${id}`);
  },

  async listObjectsByBuilding(buildingId: string): Promise<SharedObject[]> {
    return api.get(`/buildings/${buildingId}/shared-objects`);
  },

  async listAvailableObjects(buildingId: string): Promise<SharedObject[]> {
    return api.get(`/buildings/${buildingId}/shared-objects/available`);
  },

  async listObjectsByCategory(
    buildingId: string,
    category: ObjectCategory,
  ): Promise<SharedObject[]> {
    return api.get(
      `/buildings/${buildingId}/shared-objects/category/${category}`,
    );
  },

  async listObjectsByOwner(ownerId: string): Promise<SharedObject[]> {
    return api.get(`/owners/${ownerId}/shared-objects`);
  },

  async updateObject(
    id: string,
    data: Partial<SharedObject>,
  ): Promise<SharedObject> {
    return api.put(`/shared-objects/${id}`, data);
  },

  async updateAvailability(
    id: string,
    status: AvailabilityStatus,
  ): Promise<SharedObject> {
    return api.put(`/shared-objects/${id}/availability`, { status });
  },

  async deleteObject(id: string): Promise<void> {
    return api.delete(`/shared-objects/${id}`);
  },

  // Loans
  async createLoan(data: CreateLoanDto): Promise<Loan> {
    return api.post("/loans", data);
  },

  async getLoanById(id: string): Promise<Loan> {
    return api.get(`/loans/${id}`);
  },

  async listLoansByObject(objectId: string): Promise<Loan[]> {
    return api.get(`/shared-objects/${objectId}/loans`);
  },

  async listLoansByBorrower(borrowerId: string): Promise<Loan[]> {
    return api.get(`/owners/${borrowerId}/loans/borrowed`);
  },

  async listLoansByLender(lenderId: string): Promise<Loan[]> {
    return api.get(`/owners/${lenderId}/loans/lent`);
  },

  async listActiveLoansByBuilding(buildingId: string): Promise<Loan[]> {
    return api.get(`/buildings/${buildingId}/loans/active`);
  },

  async approveLoan(id: string): Promise<Loan> {
    return api.put(`/loans/${id}/approve`, {});
  },

  async startLoan(id: string): Promise<Loan> {
    return api.put(`/loans/${id}/start`, {});
  },

  async returnLoan(
    id: string,
    conditionAtReturn: ObjectCondition,
  ): Promise<Loan> {
    return api.put(`/loans/${id}/return`, {
      condition_at_return: conditionAtReturn,
    });
  },

  async cancelLoan(id: string): Promise<Loan> {
    return api.put(`/loans/${id}/cancel`, {});
  },

  async rateBorrower(id: string, rating: number): Promise<Loan> {
    return api.put(`/loans/${id}/rate-borrower`, { rating });
  },

  async rateLender(id: string, rating: number): Promise<Loan> {
    return api.put(`/loans/${id}/rate-lender`, { rating });
  },

  async getOverdueLoans(buildingId: string): Promise<Loan[]> {
    return api.get(`/buildings/${buildingId}/loans/overdue`);
  },

  // Statistics
  async getOwnerSharingStats(ownerId: string): Promise<any> {
    return api.get(`/owners/${ownerId}/sharing-stats`);
  },

  async getBuildingSharingStats(buildingId: string): Promise<any> {
    return api.get(`/buildings/${buildingId}/sharing-stats`);
  },
};
