import { api } from "../api";
import type { components } from "../../types/api";

/**
 * Object Sharing Library API Client
 * Community tool/object lending system
 *
 * `ObjectCategory` and `ObjectCondition` are re-exported from the
 * auto-generated api.d.ts (STORY-P7-704). `ObjectCategory` maps to the
 * backend schema `SharedObjectCategory`. `AvailabilityStatus` and
 * `LoanStatus` have no backend counterpart yet — they remain hand-written.
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

// Backend schema is named `SharedObjectCategory`.
export type ObjectCategory = components["schemas"]["SharedObjectCategory"];
export const ObjectCategory = {
  Tools: "Tools" as const,
  Books: "Books" as const,
  Electronics: "Electronics" as const,
  Sports: "Sports" as const,
  Gardening: "Gardening" as const,
  Kitchen: "Kitchen" as const,
  Baby: "Baby" as const,
  Other: "Other" as const,
} satisfies Record<string, ObjectCategory>;

export type ObjectCondition = components["schemas"]["ObjectCondition"];
export const ObjectCondition = {
  Excellent: "Excellent" as const,
  Good: "Good" as const,
  Fair: "Fair" as const,
  Used: "Used" as const,
} satisfies Record<string, ObjectCondition>;

// No backend counterpart — hand-written enum.
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

  async markAvailable(id: string): Promise<SharedObject> {
    return api.post(`/shared-objects/${id}/mark-available`, {});
  },

  async markUnavailable(id: string): Promise<SharedObject> {
    return api.post(`/shared-objects/${id}/mark-unavailable`, {});
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
