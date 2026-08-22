// Story B2 (Phase B FE) — client API MagicLinks.
//
// Endpoints backend (cf. `api.d.ts` regen B0 — `8cab49f` sur feature/dev) :
//   - POST   /magic-links              → issueMagicLink(req)  → IssuedMagicLink
//   - GET    /c/{token}                → endpoint PUBLIC, n'est PAS consommé
//                                        depuis le syndic — la PWA contractor
//                                        l'appelle directement (cf. Story 3.3).
//
// Types tirés de `api.d.ts` (single source of truth — pas de duplication
// maison). Si TS râle, c'est que B0 n'a pas posé le bon utoipa schema
// (cf. architecture.md §6 invariants).
//
// Pas de `cast as` sur les payloads. Le wrapper `api` (lib/api.ts) gère :
//   - Authorization Bearer auto
//   - language header
//   - toast d'erreur auto + mapping 401/403/429/500
//
// IMPORTANT — INV-FE5 (token sensitivity) :
//   Le token brut renvoyé par `issueMagicLink` ne doit JAMAIS être stocké en
//   `localStorage` / `sessionStorage` côté composant appelant — uniquement
//   dans un `$state` local du composant qui disparaît au unmount.
//   Cf. stories.md §B2 Gotcha #2.

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts` (regen openapi) — single source of truth
// -----------------------------------------------------------------------------

/** Payload d'émission (cf. backend `IssueMagicLinkRequest`). */
export type IssueMagicLinkRequest =
  components["schemas"]["IssueMagicLinkRequest"];

// -----------------------------------------------------------------------------
// Le schema response n'est pas (encore) explicitement publié par utoipa pour
// l'endpoint `/magic-links` (cf. `IssueMagicLinkResponse` côté backend
// magic_link_handlers.rs ligne 31 — pas dans `components/schemas`). On
// déclare le shape ICI en MIROIR EXACT du backend pour ne pas perdre de
// typing — si le backend change, le compilateur TS échouera dès qu'on
// touchera ce code côté composant.
//
// Follow-up B0bis si besoin : ajouter `body = IssueMagicLinkResponse` dans
// `#[utoipa::path( responses(...))]` côté backend pour générer le schema.
// -----------------------------------------------------------------------------

/** Réponse d'émission (token + métadonnées) — mirror backend ligne 31. */
export interface IssuedMagicLink {
  /** UUID du MagicLink (pas le token). */
  id: string;
  /** Token brut, single-use, à n'afficher qu'UNE FOIS au syndic. */
  token: string;
  /** ISO 8601 — date d'expiration. */
  expires_at: string;
  /** "ticket" | "quote" | "invoice" | "contractor_evaluation". */
  scope_kind: string;
  /** UUID de la ressource scopée. */
  scope_id: string;
}

// -----------------------------------------------------------------------------
// Scope kinds — aligné avec backend `MagicLinkScopeKind`
// (cf. backend/domain/entities/magic_link.rs).
// -----------------------------------------------------------------------------

export const MAGIC_LINK_SCOPE_KINDS = [
  "ticket",
  "quote",
  "invoice",
  "contractor_evaluation",
] as const;

export type MagicLinkScopeKind = (typeof MAGIC_LINK_SCOPE_KINDS)[number];

// -----------------------------------------------------------------------------
// Bornes `expires_in_seconds` — alignées backend Story 3.2.
// (1 minute → 30 jours)
// -----------------------------------------------------------------------------

/** 60 secondes — borne basse côté backend (422 si <). */
export const MAGIC_LINK_MIN_EXPIRES_IN_SECONDS = 60;
/** 30 * 24 * 3600 secondes — borne haute côté backend (422 si >). */
export const MAGIC_LINK_MAX_EXPIRES_IN_SECONDS = 30 * 24 * 3600;
/** Default 7 jours — cf. stories.md §B2 wireframe. */
export const MAGIC_LINK_DEFAULT_EXPIRES_IN_SECONDS = 7 * 24 * 3600;

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Émet un nouveau lien magique pour un destinataire (syndic / superadmin only).
 *
 * `expires_in_seconds` borné côté backend [60, 30 * 24 * 3600] (1 min → 30j) :
 *  - 422 si < 60 ou > 30j (INV — Story 3.2).
 *  - 422 `MagicLinkSelfIssue` si subject_user_id === issuer (INV-13).
 *
 * Toast automatique côté `api.ts` pour 401/403/429/5xx.
 */
export async function issueMagicLink(
  req: IssueMagicLinkRequest,
): Promise<IssuedMagicLink> {
  return api.post<IssuedMagicLink>("/magic-links", req);
}
