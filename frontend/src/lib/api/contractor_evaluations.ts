// Story B8 (Phase B FE) — client API ContractorEvaluation.
//
// Parent BE story 3.9 — ContractorEvaluation (commit `c53a7e1`) :
// évaluation contractor après prestation, append-only (INV-24), gated par
// TechnicalSpec préalable signée (INV-21 — status Approved).
//
// Endpoints backend (cf. `api.d.ts` regen B0 — lignes 576..615) :
//   - POST  /contractor-evaluations                          → createEvaluation
//   - GET   /contractor-evaluations/{id}                     → getEvaluation
//   - GET   /contractors/{contractor_user_id}/evaluations    → listForContractor
//
// INV exposés en UI (gating client AVANT POST, redondant avec backend) :
//   - INV-21 BE : TechnicalSpec sélectionnée DOIT être status="Approved".
//     L'UI filtre côté client après `listSpecs()` (cf. stories.md §B8 Gotcha
//     "filtre côté FE après GET /technical-specs?status=approved").
//   - INV-22 BE : `evaluator_user_id != contractor_user_id` (anti
//     self-evaluation). Le composant FE désactive le submit si l'utilisateur
//     courant === contractor sélectionné. AC @security stories.md §B8.
//   - INV-24 BE : append-only — pas d'edit ni de delete. Aucun bouton "Modifier"
//     / "Supprimer" sur les évaluations existantes (cf. ContractorReputation).
//   - Scores : entiers 1..=5 inclus. Le composant ScoreInput borne nativement
//     via radios (impossible de soumettre 0 ou 6 via UI).
//   - Comment : 10..=2000 chars après trim. AC @negative stories.md §B8.
//
// Types tirés DIRECTEMENT de `api.d.ts` (single source of truth — pas de
// duplication maison). Si TS râle, c'est que B0 n'a pas posé le bon utoipa
// schema (cf. architecture.md §6 invariants).
//
// Pas de `cast as` sur les payloads. Le wrapper `api` (lib/api.ts) gère :
//   - Authorization Bearer auto
//   - language header
//   - toast d'erreur auto + mapping 401/403/422/429/5xx

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts`
// -----------------------------------------------------------------------------

export type ContractorEvaluationDto =
  components["schemas"]["ContractorEvaluationDto"];
export type CreateContractorEvaluationRequest =
  components["schemas"]["CreateContractorEvaluationRequest"];
export type EvaluationScoresDto = components["schemas"]["EvaluationScoresDto"];
export type EvaluationScoresOutDto =
  components["schemas"]["EvaluationScoresOutDto"];

// -----------------------------------------------------------------------------
// Constantes métier — alignées backend Story 3.9
// -----------------------------------------------------------------------------

/** Bornes scores (cf. stories.md §B8 AC @negative — score 0 ou 6 → impossible
 *  via UI grâce au ScoreInput atomique borné 1..=5). */
export const SCORE_MIN = 1;
export const SCORE_MAX = 5;

/** Bornes comment — alignées backend + AC @negative stories.md §B8. */
export const EVAL_MIN_COMMENT_LENGTH = 10;
export const EVAL_MAX_COMMENT_LENGTH = 2000;

/** Dimensions de scoring (5 critères — cf. EvaluationScoresDto). Ordre métier
 *  : qualité technique en premier, overall en dernier (synthèse). */
export const SCORE_DIMENSIONS = [
  "quality",
  "timeliness",
  "communication",
  "cost_compliance",
  "overall",
] as const;

export type ScoreDimension = (typeof SCORE_DIMENSIONS)[number];

/** Libellés FR pour chaque dimension (i18n FR-first cf. CRITICAL.md §FR/NL/EN/DE
 *  — la i18n NL/EN/DE viendra avec Story B12+). */
export const SCORE_DIMENSION_LABELS_FR: Record<ScoreDimension, string> = {
  quality: "Qualité technique",
  timeliness: "Respect des délais",
  communication: "Communication",
  cost_compliance: "Respect du budget",
  overall: "Note globale",
};

// -----------------------------------------------------------------------------
// Helpers — validation client AVANT POST
// -----------------------------------------------------------------------------

/** True si tous les scores sont entiers dans 1..=5 inclus. */
export function isValidScores(s: EvaluationScoresDto): boolean {
  return SCORE_DIMENSIONS.every((dim) => {
    const v = s[dim];
    return Number.isInteger(v) && v >= SCORE_MIN && v <= SCORE_MAX;
  });
}

/** Comment trimmé dans la fenêtre 10..=2000 chars. */
export function isValidComment(comment: string): boolean {
  const len = [...comment.trim()].length;
  return len >= EVAL_MIN_COMMENT_LENGTH && len <= EVAL_MAX_COMMENT_LENGTH;
}

/** Calcule la moyenne d'un dimension donné sur une liste d'évaluations.
 *  Retourne `null` si la liste est vide (pas de score moyen = pas d'info,
 *  pas un "0" trompeur).
 *
 *  Note : on accepte aussi bien `EvaluationScoresDto` que `EvaluationScoresOutDto`
 *  (mêmes champs côté schéma — différence sémantique : Out vient du backend en
 *  réponse, Dto en input, mais structure identique). */
export function averageScore(
  evals: readonly ContractorEvaluationDto[],
  dim: ScoreDimension,
): number | null {
  if (evals.length === 0) return null;
  const sum = evals.reduce((acc, e) => acc + e.scores[dim], 0);
  return sum / evals.length;
}

/** Formatte une moyenne pour affichage UI : "4.2/5" ou "—" si null. */
export function formatAverage(avg: number | null): string {
  if (avg === null) return "—";
  return `${avg.toFixed(1)}/5`;
}

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Crée une nouvelle ContractorEvaluation. Append-only — pas de update/delete.
 *
 * Backend :
 *   - 201 → DTO renvoyée.
 *   - 400 → payload mal formé.
 *   - 403 → user ≠ syndic / superadmin.
 *   - 404 → TechnicalSpec introuvable.
 *   - 422 → TechnicalSpec pas en status Approved, OU self-evaluation
 *           (evaluator === contractor).
 */
export async function createEvaluation(
  req: CreateContractorEvaluationRequest,
): Promise<ContractorEvaluationDto> {
  return api.post<ContractorEvaluationDto>("/contractor-evaluations", req);
}

/**
 * Récupère une évaluation par ID. 404 si inconnue, 403 si hors scope.
 */
export async function getEvaluation(
  id: string,
): Promise<ContractorEvaluationDto> {
  return api.get<ContractorEvaluationDto>(
    `/contractor-evaluations/${encodeURIComponent(id)}`,
  );
}

/**
 * Liste les évaluations d'un contractor (newest first côté backend).
 * Page publique au sein de l'org (cf. ContractorReputation).
 */
export async function listForContractor(
  contractorUserId: string,
): Promise<ContractorEvaluationDto[]> {
  return api.get<ContractorEvaluationDto[]>(
    `/contractors/${encodeURIComponent(contractorUserId)}/evaluations`,
  );
}
