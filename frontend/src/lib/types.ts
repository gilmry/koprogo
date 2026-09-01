import type { components } from "../types/api";

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
  first_name: string;
  last_name: string;
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
  acp_id: string;
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

  // Story 1.4 — FR11/FR12/FR23 : conformity metrics exposed by GET /buildings/{id}.
  // `quota_sum` and `quota_delta` are Decimal-as-string (NEVER parseFloat).
  units_count?: number;
  quota_sum?: string;
  is_conformant?: boolean;
  quota_delta?: string;
}

// Owner interface
export interface Owner {
  id: string;
  organization_id?: string;
  user_id?: string; // Link to User account (for portal access)
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
  address?: string;
  city?: string;
  postal_code?: string;
  country?: string;
  created_at?: string;
  updated_at?: string;
}

// UnitOwner interface (junction table for many-to-many relationship)
export interface UnitOwner {
  id: string;
  unit_id: string;
  owner_id: string;
  // `string`, PAS `number` : `ownership_percentage` est un `Decimal` cote Rust
  // et `rust_decimal` le serialise en STRING JSON (ADR-0008, meme regle que
  // `total_voting_power_*`). Le declarer `number` faisait concatener au lieu
  // d'additionner partout ou il passe dans un `+`.
  ownership_percentage: string;
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
// Branché sur le contrat plutôt que recopié.
//
// La version manuscrite déclarait `quota: number`. Le backend le sérialise en
// CHAÎNE (`Decimal`, ADR-0008), ce que le contrat dit désormais explicitement :
// `quota: string`. Ce seul mensonge de type est la cause du défaut F14 du
// rapport du 2026-09-01 — `units.reduce((s, u) => s + u.quota, 0)` compilait
// sans broncher, alors que `+` concatène des chaînes : le total des tantièmes
// affichait « NaN/1000èmes », et l'indicateur de conformité des quotités
// comparait NaN, donc annonçait « quotités correctes » quel que soit
// l'encodage réel.
//
// Avec le type importé, la même ligne ne compile plus.
export type Unit = components["schemas"]["UnitResponseDto"] & {
  // Enrichissement côté client : la liste des détenteurs, chargée séparément
  // depuis `/unit-owners`. N'existe pas dans la réponse de `/units`.
  owners?: UnitOwner[];
};

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
  payment_status: "pending" | "paid" | "overdue" | "cancelled";
  approval_status?:
    "draft" | "pending_approval" | "approved" | "rejected" | null;
  paid_date?: string;
  supplier?: string;
  invoice_number?: string;
  created_at?: string;

  // Decomposition HT/TVA. Ces champs etaient renvoyes par l'API mais absents
  // de ce type : la fiche depense ne pouvait afficher que le TTC.
  //
  // Type `string | number` et non `number` : ce sont des `Decimal` cote Rust,
  // serialises en CHAINE (ADR-0008). Les declarer `number` mentait sur le
  // contenu reel et laissait passer les sommes `+` qui concatenent au lieu
  // d'additionner. Passer par `toNumber()` avant tout calcul.
  amount_excl_vat?: string | number | null;
  vat_rate?: string | number | null;
  vat_amount?: string | number | null;
  amount_incl_vat?: string | number | null;
  account_code?: string | null;
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
  // `string`, PAS `number` : `ownership_percentage` est un `Decimal` cote Rust
  // et `rust_decimal` le serialise en STRING JSON (ADR-0008, meme regle que
  // `total_voting_power_*`). Le declarer `number` faisait concatener au lieu
  // d'additionner partout ou il passe dans un `+`.
  ownership_percentage: string;
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
// ========================================
// Board of Directors Types
// ========================================

export interface BoardMemberResponse {
  id: string;
  owner_id: string;
  building_id: string;
  position: string;
  mandate_start: string;
  mandate_end: string;
  elected_by_meeting_id: string;
  is_active: boolean;
  days_remaining: number;
  expires_soon: boolean;
}

export interface BoardDecisionResponse {
  id: string;
  building_id: string;
  meeting_id: string;
  subject: string;
  decision_text: string;
  deadline?: string;
  status: "pending" | "in_progress" | "completed" | "overdue" | "cancelled";
  notes?: string;
  created_at: string;
  updated_at: string;
}

export interface DecisionStats {
  building_id: string;
  total_decisions: number;
  pending: number;
  in_progress: number;
  completed: number;
  overdue: number;
  cancelled: number;
}

export type DeadlineUrgency = "critical" | "high" | "medium";

export interface DeadlineAlert {
  decision_id: string;
  subject: string;
  deadline: string;
  days_remaining: number;
  urgency: DeadlineUrgency;
}

export interface BoardDashboardResponse {
  my_mandate?: BoardMemberResponse;
  decisions_stats: DecisionStats;
  overdue_decisions: BoardDecisionResponse[];
  upcoming_deadlines: DeadlineAlert[];
}
