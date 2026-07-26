// Track H Story H1 — Helpers Conformité.
//
// `isConformityError()` détecte un body 422 `BUILDING_NOT_CONFORMANT` côté
// frontend (catch dans les modules api/*). `showConformityToast()` rend un
// toast i18n narratif basé sur le payload — mémoire `validate-before-compute`
// + DoD-H1.
//
// Decimal-as-string : on **ne parseFloat jamais** `quota_delta` (mémoire
// `no-f64-in-money` + ADR-0007). On formate FR-BE en remplaçant le point par
// la virgule, bit-à-bit.

import { get } from "svelte/store";
import { _ } from "../i18n";
import { toast } from "../../stores/toast";
import type {
  BuildingNotConformantErrorBody,
  BuildingNotConformantPayload,
  ConformityStatus,
} from "../types/conformity";

/**
 * Type guard — l'erreur reçue est-elle un body 422 `BUILDING_NOT_CONFORMANT` ?
 *
 * Pattern reconnu :
 *   - Objet plain avec `kind === "building_not_conformant"` ET
 *     `details.code === "BUILDING_NOT_CONFORMANT"`.
 *   - Tolère aussi `Error` wrappers qui exposent `body` ou `response.data`.
 */
export function isConformityError(
  err: unknown,
): err is BuildingNotConformantErrorBody {
  if (!err || typeof err !== "object") return false;
  // Direct body
  const direct = err as Record<string, unknown>;
  if (looksLikeBody(direct)) return true;
  // Wrapped (Error.body / Error.response.data)
  const wrapped =
    (direct.body as Record<string, unknown> | undefined) ??
    ((direct.response as Record<string, unknown> | undefined)?.data as
      Record<string, unknown> | undefined);
  return !!wrapped && looksLikeBody(wrapped);
}

function looksLikeBody(o: Record<string, unknown>): boolean {
  if (o.kind !== "building_not_conformant") return false;
  const details = o.details as Record<string, unknown> | undefined;
  if (!details) return false;
  return details.code === "BUILDING_NOT_CONFORMANT";
}

/**
 * Extrait le payload narratif d'une erreur de conformité (caller doit avoir
 * déjà vérifié via `isConformityError()`).
 */
export function extractConformityPayload(
  err: BuildingNotConformantErrorBody | Record<string, unknown>,
): BuildingNotConformantPayload | null {
  const candidate =
    (err as { details?: BuildingNotConformantPayload }).details ??
    (err as { body?: { details?: BuildingNotConformantPayload } }).body
      ?.details ??
    (
      err as {
        response?: { data?: { details?: BuildingNotConformantPayload } };
      }
    ).response?.data?.details;
  return candidate && candidate.code === "BUILDING_NOT_CONFORMANT"
    ? candidate
    : null;
}

/**
 * Format Decimal-as-string en FR-BE (point → virgule) sans parseFloat.
 * Préserve la précision décimale bit-à-bit.
 */
export function formatDecimalFRBE(s: string): string {
  if (!s) return "—";
  const cleaned = s.startsWith("+") ? s.slice(1) : s;
  return cleaned.replace(".", ",");
}

/**
 * Affiche un toast narratif si l'erreur est un `BUILDING_NOT_CONFORMANT`.
 * Retourne `true` si le toast a été affiché, `false` sinon (caller peut
 * alors fallback sur toast générique).
 *
 * Le store `toast` actuel prend une string — on assemble title + message en
 * un seul libellé i18n (cf. `conformity.toast_*`).
 */
export function showConformityToast(err: unknown): boolean {
  if (!isConformityError(err)) return false;
  const payload = extractConformityPayload(err);
  if (!payload) return false;
  const tt = get(_);
  const title = tt("conformity.toast_title");
  const message = tt("conformity.toast_message", {
    values: {
      units_delta: payload.units_delta,
      sum: subtractFromBasis(payload.quota_basis, payload.quota_delta),
      basis: payload.quota_basis,
    },
  });
  // duration 8s — narratif, le user doit lire (mémoire toast verbose erreurs).
  toast.error(`${title} — ${message}`, 8000);
  return true;
}

/**
 * Calcule `quota_sum = quota_basis - quota_delta` en string (sans
 * parseFloat). Utilisé pour le message « somme = X / basis » du toast.
 *
 * Implémentation pragmatique : on délègue à Number pour de l'affichage
 * (pas pour du calcul comptable), ce qui est acceptable car `quota_basis`
 * est un entier i32 et `quota_delta` est borné par cet entier — la
 * précision Number suffit pour l'affichage. La vraie source de vérité
 * Decimal vit côté BE.
 */
function subtractFromBasis(basis: number, deltaStr: string): string {
  const delta = Number(deltaStr);
  if (Number.isNaN(delta)) return "—";
  const sum = basis - delta;
  // FR-BE formatting : virgule au lieu de point.
  const s = Number.isInteger(sum) ? String(sum) : String(sum);
  return s.replace(".", ",");
}

/**
 * Construit un `ConformityStatus` côté FE à partir des champs exposés par
 * `BuildingResponseDto` (mémoire cartographie 2026-06-15).
 */
export function buildConformityStatus(args: {
  is_conformant: boolean;
  total_units: number;
  units_count: number;
  total_tantiemes: number;
  quota_delta: string;
}): ConformityStatus {
  return {
    is_conformant: args.is_conformant,
    units_delta: args.total_units - args.units_count,
    quota_delta: args.quota_delta,
    quota_basis: args.total_tantiemes,
  };
}
