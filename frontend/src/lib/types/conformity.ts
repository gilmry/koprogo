// Track H Story H1 — Types Conformité immeuble.
//
// Le BE expose `is_conformant + units_count + total_units + quota_sum +
// quota_delta` sur `BuildingResponseDto` (cf. cartographie 2026-06-15). Le
// composant atomique `<ConformityBanner>` consomme ce sous-ensemble via
// `ConformityStatus`.
//
// Le BE expose aussi le payload 422 `BUILDING_NOT_CONFORMANT` sur les
// use-cases validate-before-compute (Track H Story H2) — typage TS dans
// `BuildingNotConformantPayload`.

/**
 * Statut conformité d'un immeuble vis-à-vis de son acte de base.
 *
 * `quota_basis` = `total_tantiemes` (1000, 10000, autre — acte de base).
 * Story H1 bug fix : `quota_basis` est **lu sur l'immeuble**, jamais hard-codé.
 *
 * `quota_delta` est sérialisé **string** (Decimal-as-string — mémoire
 * `no-f64-in-money` + ADR-0007). Convention BE :
 * `quota_delta = quota_basis - quota_sum` → positif = manque, négatif = surplus.
 */
export interface ConformityStatus {
  /** L'immeuble est-il conformant (count==total && SUM(quota)==basis) ? */
  is_conformant: boolean;
  /** `total_units - units_count` (positif = lots manquants). */
  units_delta: number;
  /** Decimal-as-string (« 0.1 », « 25 », « -50 »). NE PAS parseFloat. */
  quota_delta: string;
  /** Acte de base de l'immeuble (1000, 10000, autre). */
  quota_basis: number;
}

/**
 * Payload 422 narratif renvoyé par les use-cases validate-before-compute
 * quand `Building::assert_conformant()` échoue. Consommé par le toast FE +
 * banner de la page concernée.
 *
 * Format BE : `error_response()` injecte `details` dans le body 422 :
 * ```json
 * { "error": "...", "kind": "building_not_conformant", "details": {
 *     "code": "BUILDING_NOT_CONFORMANT",
 *     "building_id": "uuid",
 *     "units_delta": 1,
 *     "quota_delta": "2.5",
 *     "quota_basis": 1000
 * } }
 * ```
 */
export interface BuildingNotConformantPayload {
  code: "BUILDING_NOT_CONFORMANT";
  building_id: string;
  units_delta: number;
  /** Decimal-as-string. */
  quota_delta: string;
  quota_basis: number;
}

/**
 * Track H Story H5 — le MÊME constat, un cran plus haut : la copropriété
 * entière ne correspond pas à son acte de base.
 *
 * `error.rs` le dit explicitement : « Même format que
 * BUILDING_NOT_CONFORMANT ». Seul l'identifiant change (`acp_id` au lieu de
 * `building_id`) ; `units_delta`, `quota_delta` et `quota_basis` sont
 * identiques, et ce sont les trois seuls champs que le toast narratif lit.
 *
 * Ce type manquait : le frontend ne reconnaissait que la variante immeuble,
 * et une refus au niveau ACP passait donc SANS toast ni bannière —
 * l'utilisateur voyait un 422 nu. Constaté le 2026-09-17 sur
 * `/call-for-funds` (#942).
 */
export interface AcpNotConformantPayload {
  code: "ACP_NOT_CONFORMANT";
  acp_id: string;
  units_delta: number;
  /** Decimal-as-string. */
  quota_delta: string;
  quota_basis: number;
}

/** L'un ou l'autre : les deux portent le même récit à l'utilisateur. */
export type ConformityPayload =
  BuildingNotConformantPayload | AcpNotConformantPayload;

/** Forme du body 422 complet (utile pour le typage du toast handler). */
export interface BuildingNotConformantErrorBody {
  error: string;
  kind: "building_not_conformant";
  details: BuildingNotConformantPayload;
}
