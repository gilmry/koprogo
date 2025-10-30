// User roles in the SaaS platform
export enum UserRole {
  SUPERADMIN = "superadmin", // Platform administrator
  SYNDIC = "syndic", // Property manager
  ACCOUNTANT = "accountant", // Accountant
  OWNER = "owner", // Co-owner
}

export interface UserRoleSummary {
  id: string;
  role: UserRole;
  organizationId?: string;
  isPrimary: boolean;
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
  is_active?: boolean;
  created_at?: string;
  roles: UserRoleSummary[];
  activeRole?: UserRoleSummary;
}

// Organization subscription plans
export enum SubscriptionPlan {
  FREE = "free",
  STARTER = "starter",
  PROFESSIONAL = "professional",
  ENTERPRISE = "enterprise",
}

// Organization interface
export interface Organization {
  id: string;
  name: string;
  slug: string;
  contact_email: string;
  contact_phone?: string;
  subscription_plan: SubscriptionPlan;
  max_buildings: number;
  max_users: number;
  is_active: boolean;
  created_at: string;
  updated_at?: string;
}

// Building interface
export interface Building {
  id: string;
  organization_id: string;
  name: string;
  address: string;
  city: string;
  postal_code: string;
  country: string;
  total_units: number;
  total_tantiemes: number; // Total shares (typically 1000 in Belgium)
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

// UnitOwner interface (junction table for many-to-many relationship)
export interface UnitOwner {
  id: string;
  unit_id: string;
  owner_id: string;
  ownership_percentage: number;
  start_date: string;
  end_date?: string;
  is_primary_contact: boolean;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
  // Populated fields (when joined)
  owner?: Owner;
  unit?: Unit;
}

// Unit interface
export interface Unit {
  id: string;
  building_id: string;
  unit_number: string;
  floor: number;
  surface_area: number;
  quota: number; // Quote-part en millièmes (déjà exprimée sur 1000, ex: 350 = 350/1000èmes)
  unit_type: "Apartment" | "Parking" | "Storage";
  owner_id?: string; // Deprecated - use unit_owners instead
  // Optional: populated owners list
  owners?: UnitOwner[];
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
  supplier?: string;
  invoice_number?: string;
  created_at?: string;
}

// Meeting interface
export interface Meeting {
  id: string;
  building_id: string;
  meeting_type: string;
  title: string;
  description?: string;
  scheduled_date: string;
  location: string;
  status: "Scheduled" | "Completed" | "Cancelled";
  agenda: string[]; // Liste des points à l'ordre du jour
  attendees_count?: number;
  created_at?: string;
  updated_at?: string;
}

// Document interface
export type DocumentType =
  | "MeetingMinutes"
  | "FinancialStatement"
  | "Invoice"
  | "Contract"
  | "Regulation"
  | "WorksQuote"
  | "Other";

export interface Document {
  id: string;
  building_id: string;
  document_type: DocumentType;
  title: string;
  description?: string | null;
  file_path: string;
  file_size: number;
  mime_type: string;
  uploaded_by: string;
  related_meeting_id?: string | null;
  related_expense_id?: string | null;
  created_at: string;
  updated_at: string;
}

export interface DocumentUploadPayload {
  buildingId: string;
  documentType: DocumentType;
  title: string;
  description?: string;
  file: File;
  uploadedBy: string;
}

export const DOCUMENT_TYPE_OPTIONS: {
  value: DocumentType;
  label: string;
}[] = [
  { value: "MeetingMinutes", label: "Procès-verbal" },
  { value: "FinancialStatement", label: "Bilan financier" },
  { value: "Invoice", label: "Facture" },
  { value: "Contract", label: "Contrat" },
  { value: "Regulation", label: "Règlement" },
  { value: "WorksQuote", label: "Devis travaux" },
  { value: "Other", label: "Autre" },
];

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

// ============================================================================
// GDPR Types (Articles 15 & 17)
// ============================================================================

export interface GdprUserData {
  id: string;
  email: string;
  first_name: string;
  last_name: string;
  organization_id?: string;
  is_active: boolean;
  is_anonymized: boolean;
  created_at: string;
}

export interface GdprOwnerData {
  id: string;
  organization_id?: string;
  user_id?: string;
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
  address: string;
  city: string;
  postal_code: string;
  country: string;
  is_anonymized: boolean;
  created_at: string;
  anonymized_at?: string;
}

export interface GdprUnitOwnershipData {
  id: string;
  unit_id: string;
  owner_id: string;
  unit_number?: string;
  building_name?: string;
  ownership_percentage: number;
  start_date: string;
  end_date?: string;
  is_primary_contact: boolean;
  is_active: boolean;
}

export interface GdprExpenseData {
  id: string;
  building_id: string;
  amount: number;
  description: string;
  due_date: string;
  paid: boolean;
  created_at: string;
}

export interface GdprDocumentData {
  id: string;
  title: string;
  document_type: string;
  file_size: number;
  created_at: string;
}

export interface GdprMeetingData {
  id: string;
  building_id: string;
  title: string;
  meeting_type: string;
  scheduled_at: string;
  status: string;
  created_at: string;
}

export interface GdprExport {
  export_date: string;
  user: GdprUserData;
  owners: GdprOwnerData[];
  units: GdprUnitOwnershipData[];
  expenses: GdprExpenseData[];
  documents: GdprDocumentData[];
  meetings: GdprMeetingData[];
  total_items: number;
}

export interface GdprEraseResponse {
  success: boolean;
  message: string;
  anonymized_at: string;
  user_id: string;
  user_email: string;
  user_first_name: string;
  user_last_name: string;
  owners_anonymized: number;
}

export interface GdprCanEraseResponse {
  can_erase: boolean;
  user_id: string;
  legal_holds: number;
}
