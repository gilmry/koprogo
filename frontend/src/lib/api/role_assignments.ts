// Story B1 (Phase B FE) — Role assignments API client.
//
// Parent BE story 3.1 (sub-rôles métier `accountant.encodeur` /
// `accountant.emetteur` / `community.moderator` + mandataires) + Story B0bis
// (REST gap — endpoints CRUD assignments — commit `8ac5a83`).
//
// Endpoints typés via `api.d.ts` (regen openapi) — pas de duplicat manuel des
// définitions, pas de `cast as` (cf. architecture.md §6 Phase B FE).
//
// Endpoints utilisés :
//   POST   /users/{user_id}/role-assignments   → assigner un sous-rôle
//   GET    /users/{user_id}/role-assignments   → lister les assignments d'un user
//   DELETE /users/{user_id}/role-assignments/{assignment_id} → révoquer
//   GET    /role-assignments?organization_id=&role= → liste admin filtrée
//
// Sécurité / multi-tenant : le backend gère le scope (superadmin / syndic
// d'organization) — pas d'enforcement côté FE. Un 403 remonte ici comme
// `Error` (cf. `api.ts` ligne 161) avec toast automatique côté `api.ts`.

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts` (regen openapi) — single source of truth
// -----------------------------------------------------------------------------

/** Réponse d'un assignment (cf. backend `UserRoleAssignmentResponse`). */
export type RoleAssignment =
  components["schemas"]["UserRoleAssignmentResponse"];

/** Payload de création d'assignment (cf. backend `AssignRoleRequest`). */
export type AssignRoleRequest = components["schemas"]["AssignRoleRequest"];

// -----------------------------------------------------------------------------
// Liste des rôles sélectionnables dans le form (Story B1 §AC @happy)
//
// Aligné avec backend Story 3.1 : sous-rôles métier + mandataires.
// Les rôles primaires `superadmin/syndic/accountant/owner` ne sont PAS
// assignables ici — ils relèvent du onboarding user, pas d'un sous-rôle.
// -----------------------------------------------------------------------------

export const ASSIGNABLE_ROLES = [
  // Comptabilité — séparation pouvoirs INV-10
  "accountant.encodeur",
  "accountant.emetteur",
  // Communauté — modération SEL / threads
  "community.moderator",
  // Mandataires juridiques / techniques
  "lawyer",
  "notary",
  "amo",
  "architect",
  "bet",
  "warden",
] as const;

export type AssignableRole = (typeof ASSIGNABLE_ROLES)[number];

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Assigner un sous-rôle à un user existant.
 *
 * @param userId UUID de l'utilisateur cible.
 * @param req payload { role, organization_id?, valid_until? }.
 * @returns le `RoleAssignment` créé (201).
 * @throws sur 400 (rôle invalide), 403 (pas superadmin/syndic), 404 (user
 *         inconnu), 409 (rôle déjà actif). Toast automatique via `api.ts`.
 */
export async function createRoleAssignment(
  userId: string,
  req: AssignRoleRequest,
): Promise<RoleAssignment> {
  return api.post<RoleAssignment>(
    `/users/${encodeURIComponent(userId)}/role-assignments`,
    req,
  );
}

/**
 * Lister les assignments actifs d'un utilisateur.
 */
export async function listForUser(userId: string): Promise<RoleAssignment[]> {
  return api.get<RoleAssignment[]>(
    `/users/${encodeURIComponent(userId)}/role-assignments`,
  );
}

/**
 * Lister les assignments filtrés (admin). Filtres optionnels par
 * `organization_id` et/ou `role`. Endpoint superadmin only (403 sinon).
 */
export async function listAssignments(filters?: {
  organization_id?: string | null;
  role?: AssignableRole | string;
}): Promise<RoleAssignment[]> {
  const params = new URLSearchParams();
  if (filters?.organization_id) {
    params.append("organization_id", filters.organization_id);
  }
  if (filters?.role) {
    params.append("role", String(filters.role));
  }
  const qs = params.toString();
  const path = qs ? `/role-assignments?${qs}` : "/role-assignments";
  return api.get<RoleAssignment[]>(path);
}

/**
 * Révoquer un assignment existant.
 *
 * @throws sur 403 (pas le droit) / 404 (assignment inconnu).
 */
export async function revokeAssignment(
  userId: string,
  assignmentId: string,
): Promise<void> {
  await api.delete<void>(
    `/users/${encodeURIComponent(userId)}/role-assignments/${encodeURIComponent(assignmentId)}`,
  );
}
