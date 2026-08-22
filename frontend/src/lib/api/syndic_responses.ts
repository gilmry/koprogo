// Story B6 (Phase B FE) — client API SyndicResponses (réponses syndic à
// un ticket, append-only).
//
// Endpoints backend (cf. `api.d.ts` regen B0 — `8cab49f` sur feature/dev) :
//   - POST /tickets/{id}/syndic-responses  → respondToTicket(...)  → SyndicResponseDto
//   - GET  /tickets/{id}/syndic-responses  → listResponsesForTicket(id) → SyndicResponseDto[]
//
// Types tirés DIRECTEMENT de `api.d.ts` (single source of truth — pas de
// duplication maison). Si TS râle, c'est que B0 n'a pas posé le bon
// utoipa schema (cf. architecture.md §6 invariants).
//
// Pas de `cast as` sur les payloads. Le wrapper `api` (lib/api.ts) gère :
//   - Authorization Bearer auto
//   - language header
//   - toast d'erreur auto + mapping 401/403/422/429/5xx
//
// IMPORTANT — INV-FE8 (append-only) :
//   Le contrat backend interdit toute mutation/suppression d'une response
//   passée. Conséquence FE : aucun composant ne doit exposer "Edit"/"Delete"
//   sur les réponses listées (cf. AC @security stories.md §B6).

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts`
// -----------------------------------------------------------------------------

/** Une réponse syndic immuable (cf. backend `SyndicResponseDto`). */
export type SyndicResponseDto = components["schemas"]["SyndicResponseDto"];
/** Payload POST (cf. backend `CreateSyndicResponseRequest`). */
export type CreateSyndicResponseRequest =
  components["schemas"]["CreateSyndicResponseRequest"];

// -----------------------------------------------------------------------------
// Action proposée — set fermé aligné avec backend (cf. stories.md §B6 schema
// + `CreateSyndicResponseRequest.action_proposed` doc :
//   `schedule_inspection`, `request_quote`, `closed_no_action`,
//   `escalated_board`, `other` — optionnel).
// -----------------------------------------------------------------------------

export const SYNDIC_RESPONSE_ACTIONS = [
  "schedule_inspection",
  "request_quote",
  "closed_no_action",
  "escalated_board",
  "other",
] as const;

export type SyndicResponseAction = (typeof SYNDIC_RESPONSE_ACTIONS)[number];

// -----------------------------------------------------------------------------
// Bornes body (cf. stories.md §B6 AC @negative) — alignées backend Story 3.7.
// -----------------------------------------------------------------------------

/** Body minimum 10 chars après trim — 422 backend en deçà. */
export const SYNDIC_RESPONSE_MIN_BODY_LENGTH = 10;
/** Body maximum 5 000 chars — 422 backend au delà. */
export const SYNDIC_RESPONSE_MAX_BODY_LENGTH = 5_000;

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Liste les réponses syndic pour un ticket — ordre chronologique (oldest first).
 *
 * Backend filtre par RBAC : owner, syndic, contractor lié au ticket → 200.
 * Hors scope → 403 (toast automatique).
 */
export async function listResponsesForTicket(
  ticketId: string,
): Promise<SyndicResponseDto[]> {
  return api.get<SyndicResponseDto[]>(
    `/tickets/${encodeURIComponent(ticketId)}/syndic-responses`,
  );
}

/**
 * Poste une nouvelle réponse syndic — append-only.
 *
 * Backend :
 *   - 201 → DTO renvoyé.
 *   - 400/422 → body < 10 chars, > 5000 chars, action_proposed invalide.
 *   - 403 → user ≠ syndic / superadmin.
 *
 * Le wrapper `api.post` propage l'erreur en `Error` côté FE (le composant
 * gère le message inline).
 */
export async function respondToTicket(
  ticketId: string,
  req: CreateSyndicResponseRequest,
): Promise<SyndicResponseDto> {
  return api.post<SyndicResponseDto>(
    `/tickets/${encodeURIComponent(ticketId)}/syndic-responses`,
    req,
  );
}
