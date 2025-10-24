// User roles in the SaaS platform
export enum UserRole {
  SUPERADMIN = "superadmin", // Platform administrator
  SYNDIC = "syndic", // Property manager
  ACCOUNTANT = "accountant", // Accountant
  OWNER = "owner", // Co-owner
}

// User type
export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  organizationId?: string; // For multi-tenant support
  buildingIds?: string[]; // Buildings the user has access to
}

// Building interface
export interface Building {
  id: string;
  name: string;
  address: string;
  city: string;
  postal_code: string;
  country: string;
  total_units: number;
  construction_year?: number;
  created_at?: string;
  updated_at?: string;
}

// Owner interface
export interface Owner {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
  created_at?: string;
}

// Unit interface
export interface Unit {
  id: string;
  building_id: string;
  unit_number: string;
  floor: number;
  surface_area: number;
  ownership_share: number;
  unit_type: "Apartment" | "Parking" | "Storage";
  owner_id?: string;
}

// Expense interface
export interface Expense {
  id: string;
  building_id: string;
  description: string;
  amount: number;
  expense_date: string;
  due_date: string;
  category:
    | "Maintenance"
    | "Repair"
    | "Insurance"
    | "Utilities"
    | "Management"
    | "Other";
  payment_status: "Pending" | "Paid" | "Overdue" | "Cancelled";
  paid_date?: string;
}

// Pagination types (matches backend PageResponse)
export interface PaginationMeta {
  current_page: number;
  per_page: number;
  total_items: number;
  total_pages: number;
  has_next: boolean;
  has_previous: boolean;
}

export interface PageResponse<T> {
  data: T[];
  pagination: PaginationMeta;
}

export interface PageRequest {
  page?: number;
  per_page?: number;
}

// Permission helpers
export const hasPermission = (
  user: User | null,
  requiredRole: UserRole,
): boolean => {
  if (!user) return false;

  const roleHierarchy = {
    [UserRole.SUPERADMIN]: 4,
    [UserRole.SYNDIC]: 3,
    [UserRole.ACCOUNTANT]: 2,
    [UserRole.OWNER]: 1,
  };

  return roleHierarchy[user.role] >= roleHierarchy[requiredRole];
};

export const canAccessBuilding = (
  user: User | null,
  buildingId: string,
): boolean => {
  if (!user) return false;
  if (user.role === UserRole.SUPERADMIN) return true;
  return user.buildingIds?.includes(buildingId) ?? false;
};
