// Story B4 (Phase B FE) — client API Role delegations.
//
// Parent BE story 3.5 (`edf171f` — UserRoleAssignment.valid_until + delegation
// use-case, INV-8 non-transitivité + max 90j) + Story B0 utoipa registrations
// (`8cab49f` — `/role-delegations` paths).
//
// Endpoints backend (cf. `api.d.ts` regen B0) :
//   POST   /role-delegations            → delegateRole(req)        → RoleDelegationResponse
//   GET    /role-delegations?subject=X  → listDelegationsOf(X?)    → RoleDelegationResponse[]
//   DELETE /role-delegations/{id}       → revokeDelegation(id)     → 204
//
// Types tirés DIRECTEMENT de `api.d.ts` (single source of truth — pas de
// duplication maison). Si TS râle, c'est que B0 n'a pas posé le bon utoipa
// schema (cf. architecture.md §6 invariants).
//
// Sécurité / multi-tenant : RBAC backend gère le scope (superadmin / syndic).
// Un 403 remonte ici comme `Error` (cf. `api.ts` ligne 161) avec toast
// automatique côté `api.ts`. `DelegationChainNotAllowed` (INV-8 non-transitivité)
// vient en 403 — le composant `RoleDelegationForm` masque déjà le CTA et
// affiche un banner avant tout POST pour éviter cette tentative.

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts` (regen openapi) — single source of truth
// -----------------------------------------------------------------------------

/** Réponse d'une délégation (cf. backend `RoleDelegationResponse`). */
export type RoleDelegationResponse =
  components["schemas"]["RoleDelegationResponse"];

/** Payload de création d'une délégation (cf. backend `DelegateRoleRequest`). */
export type DelegateRoleRequest = components["schemas"]["DelegateRoleRequest"];

// -----------------------------------------------------------------------------
// Liste des rôles délégables — sous-ensemble des rôles primaires/sub-rôles
// (cf. stories.md §B4 — Syndic peut déléguer son rôle, ou un board member peut
// déléguer une fonction encadrée). On exclut `superadmin` (jamais délégué) et
// `accountant.*` (séparation des pouvoirs INV-10 incompatible avec délégation).
// -----------------------------------------------------------------------------

export const DELEGABLE_ROLES = [
  "syndic",
  "owner",
  "community.moderator",
] as const;

export type DelegableRole = (typeof DELEGABLE_ROLES)[number];

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Déléguer temporairement un rôle à un user existant.
 *
 * @param req payload { target_user_id, role, organization_id?, valid_until }.
 * @returns la `RoleDelegationResponse` créée (201).
 * @throws sur 400 (validation), 403 (caller cannot delegate this role —
 *         incl. `DelegationChainNotAllowed` INV-8), 409 (target already holds
 *         the role). Toast automatique via `api.ts`.
 */
export async function delegateRole(
  req: DelegateRoleRequest,
): Promise<RoleDelegationResponse> {
  return api.post<RoleDelegationResponse>("/role-delegations", req);
}

/**
 * Liste les délégations actives concernant un subject user.
 *
 * @param subjectUserId UUID optionnel — défaut = caller (utile pour détecter
 *        si le current user a HÉRITÉ son rôle via une délégation, cf.
 *        non-transitivité INV-8 + UI banner).
 * @throws 403 si l'appelant n'a pas le droit de voir les délégations du
 *         subject (RBAC backend).
 */
export async function listDelegationsOf(
  subjectUserId?: string,
): Promise<RoleDelegationResponse[]> {
  const path = subjectUserId
    ? `/role-delegations?subject=${encodeURIComponent(subjectUserId)}`
    : "/role-delegations";
  return api.get<RoleDelegationResponse[]>(path);
}

/**
 * Révoquer une délégation avant son expiration naturelle.
 *
 * Backend renvoie 204 No Content. Le wrapper `api.delete` retourne `undefined`.
 *
 * @throws 403 (pas le droit) / 404 (délégation inconnue).
 */
export async function revokeDelegation(id: string): Promise<void> {
  await api.delete<void>(`/role-delegations/${encodeURIComponent(id)}`);
}
