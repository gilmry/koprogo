// Story B3 (Phase B FE) — client API Mandates.
//
// Endpoints backend (cf. `api.d.ts` regen B0 — `8ac5a83` sur feature/dev) :
//   - POST   /mandates                 → issueMandate(req)   → MandateResponse
//   - GET    /mandates                 → listMandates()      → MandateResponse[]
//   - GET    /mandates/{id}            → getMandate(id)      → MandateResponse
//   - POST   /mandates/{id}/revoke     → revokeMandate(id)   → 204
//
// Types tirés DIRECTEMENT de `api.d.ts` (single source of truth — pas de
// duplication maison). Si TS râle, c'est que B0 n'a pas posé le bon
// utoipa schema (cf. architecture.md §6 invariants).
//
// Pas de `cast as` sur les payloads. Le wrapper `api` (lib/api.ts) gère :
//   - Authorization Bearer auto
//   - language header
//   - toast d'erreur auto + mapping 401/403/429/500
//   - 204 → undefined (utile pour revokeMandate)

import { api } from "../api";
import type { components } from "../../types/api";

// Re-exports typés depuis api.d.ts (regen via `npm run types:generate`).
export type MandateResponse = components["schemas"]["MandateResponse"];
export type IssueMandateRequest = components["schemas"]["IssueMandateRequest"];

/**
 * Émet un nouveau mandat (syndic / superadmin seulement — RBAC backend).
 *
 * `valid_until` : ISO 8601 obligatoire. Backend rejette en 422 si > 5 ans
 * (INV-14) ou si `subject_user_id === issuer` (INV-15).
 */
export async function issueMandate(
  req: IssueMandateRequest,
): Promise<MandateResponse> {
  return api.post<MandateResponse>("/mandates", req);
}

/**
 * Liste les mandats actifs pour le subject user (par défaut : user courant).
 *
 * Côté table syndic (`MandateList`) on liste les mandats émis dans son scope.
 * Filtrage RBAC backend — 403 si scope cross-tenant.
 */
export async function listMandates(): Promise<MandateResponse[]> {
  return api.get<MandateResponse[]>("/mandates");
}

/**
 * Récupère un mandat par ID. 404 si inconnu, 403 si hors scope.
 */
export async function getMandate(id: string): Promise<MandateResponse> {
  return api.get<MandateResponse>(`/mandates/${encodeURIComponent(id)}`);
}

/**
 * Révoque un mandat avant expiration naturelle.
 *
 * Backend renvoie 204 No Content (mis à jour le `revoked_at`).
 * Le wrapper `api.post` retourne `undefined` pour les 204 (cf. lib/api.ts).
 */
export async function revokeMandate(id: string): Promise<void> {
  await api.post<void>(`/mandates/${encodeURIComponent(id)}/revoke`, {});
}
