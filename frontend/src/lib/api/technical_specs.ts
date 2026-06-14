// Story B7 (Phase B FE) — client API TechnicalSpecs.
//
// Parent BE story 3.8 — TechnicalSpec versionnable (semver strict, signatures
// par (user, role) — INV unique, bump major invalide signatures).
//
// Endpoints backend (cf. `api.d.ts` regen B0) :
//   - GET    /technical-specs                       → listSpecs()        → TechnicalSpecDto[]
//   - POST   /technical-specs                       → createSpec(req)    → TechnicalSpecDto
//   - GET    /technical-specs/{id}                  → getSpec(id)        → TechnicalSpecDto
//   - POST   /technical-specs/{id}/bump             → bumpVersion(id,r)  → TechnicalSpecDto
//   - POST   /technical-specs/{id}/submit           → submitForSignatures(id) → TechnicalSpecDto
//   - POST   /technical-specs/{id}/signatures       → signSpec(id, r)    → TechnicalSpecSignatureDto
//   - GET    /technical-specs/{id}/signatures       → listSignatures(id) → TechnicalSpecSignatureDto[]
//
// Types tirés DIRECTEMENT de `api.d.ts` (single source of truth — pas de
// duplication maison). Si TS râle, c'est que B0 n'a pas posé le bon utoipa
// schema (cf. architecture.md §6 invariants).
//
// Pas de `cast as` sur les payloads. Le wrapper `api` (lib/api.ts) gère :
//   - Authorization Bearer auto
//   - language header
//   - toast d'erreur auto + mapping 401/403/422/429/5xx
//
// INV BE TechnicalSpec (cf. stories.md §B7) :
//   - SemVer strict : "major.minor.patch" — pas de `v`-prefix, pas de
//     pre-release / build metadata.
//   - Signatures uniques par (user, role).
//   - Bump major (1.x.y → 2.0.0) invalide les signatures précédentes
//     (re-signature requise — warning UI côté FE).
//   - Bump minor/patch préserve les signatures.
//   - Status workflow : Draft → PendingSignatures → Approved → Superseded.

import { api } from "../api";
import type { components } from "../../types/api";

// -----------------------------------------------------------------------------
// Types réexportés depuis `api.d.ts`
// -----------------------------------------------------------------------------

export type TechnicalSpecDto = components["schemas"]["TechnicalSpecDto"];
export type TechnicalSpecSignatureDto =
  components["schemas"]["TechnicalSpecSignatureDto"];
export type CreateTechnicalSpecRequest =
  components["schemas"]["CreateTechnicalSpecRequest"];
export type BumpTechnicalSpecRequest =
  components["schemas"]["BumpTechnicalSpecRequest"];
export type SignTechnicalSpecRequest =
  components["schemas"]["SignTechnicalSpecRequest"];

// -----------------------------------------------------------------------------
// Constantes métier — alignées backend Story 3.8
// -----------------------------------------------------------------------------

/** Status workflow (string côté DTO car backend renvoie un libellé libre). */
export const TECH_SPEC_STATUSES = [
  "Draft",
  "PendingSignatures",
  "Approved",
  "Superseded",
] as const;

export type TechSpecStatus = (typeof TECH_SPEC_STATUSES)[number];

/** Rôles signataires autorisés (cf. backend SignatoryRole enum). */
export const SIGNATORY_ROLES = [
  "syndic",
  "amo",
  "lawyer",
  "architect",
  "acp_representative",
] as const;

export type SignatoryRole = (typeof SIGNATORY_ROLES)[number];

/** Rôles "mandataire" qui REQUIÈRENT un mandate_id actif pour signer
 *  (cf. Story 3.4 chain — stories.md §B7 AC @security). */
export const MANDATARY_ROLES: ReadonlyArray<SignatoryRole> = [
  "amo",
  "lawyer",
  "architect",
] as const;

/** Bornes description — alignées backend + AC @negative stories.md §B7. */
export const TECH_SPEC_MIN_DESCRIPTION_LENGTH = 50;
export const TECH_SPEC_MAX_DESCRIPTION_LENGTH = 5000;

// -----------------------------------------------------------------------------
// Helpers semver — validation client AVANT POST (gating — cf. AC @edge §B7)
// -----------------------------------------------------------------------------

/**
 * Regex SemVer strict (cf. stories.md §B7 AC @edge) :
 *   - "1.0.0" OK
 *   - "v1.0.0" REJETÉ (pas de prefix v)
 *   - "1.0.0-rc1" REJETÉ (pas de pre-release)
 *   - "1.0" REJETÉ (3 segments obligatoires)
 *   - "1.0.0+build" REJETÉ (pas de build metadata)
 *
 * Pattern : `^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$`
 * Refuse les zéros de tête (sauf le seul "0") — cohérent avec SemVer 2.0.
 */
const SEMVER_STRICT = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

export function isValidSemver(v: string): boolean {
  return SEMVER_STRICT.test(v);
}

export type SemverParts = {
  major: number;
  minor: number;
  patch: number;
};

export function parseSemver(v: string): SemverParts | null {
  const m = SEMVER_STRICT.exec(v);
  if (!m) return null;
  return {
    major: Number(m[1]),
    minor: Number(m[2]),
    patch: Number(m[3]),
  };
}

/** Compare deux versions semver. Retourne <0 si a<b, 0 si égal, >0 si a>b. */
export function compareSemver(a: SemverParts, b: SemverParts): number {
  if (a.major !== b.major) return a.major - b.major;
  if (a.minor !== b.minor) return a.minor - b.minor;
  return a.patch - b.patch;
}

/** Détecte un bump MAJOR (a.major > b.major). Utilisé pour avertir
 *  l'utilisateur que les signatures précédentes seront invalidées. */
export function isMajorBump(prev: string, next: string): boolean {
  const p = parseSemver(prev);
  const n = parseSemver(next);
  if (!p || !n) return false;
  return n.major > p.major;
}

// -----------------------------------------------------------------------------
// API functions
// -----------------------------------------------------------------------------

/**
 * Liste les TechnicalSpecs accessibles à l'utilisateur courant.
 *
 * Backend filtre par RBAC (syndic / superadmin / mandataires des ACPs).
 * Hors scope → 403 (toast automatique).
 */
export async function listSpecs(): Promise<TechnicalSpecDto[]> {
  return api.get<TechnicalSpecDto[]>("/technical-specs");
}

/**
 * Crée une nouvelle TechnicalSpec en status `Draft`.
 *
 * Backend :
 *   - 201 → DTO renvoyé.
 *   - 400/422 → deliverables vide, version mal formée, description trop courte.
 *   - 403 → user ≠ syndic / superadmin.
 */
export async function createSpec(
  req: CreateTechnicalSpecRequest,
): Promise<TechnicalSpecDto> {
  return api.post<TechnicalSpecDto>("/technical-specs", req);
}

/**
 * Récupère une TechnicalSpec par ID. 404 si inconnue, 403 si hors scope.
 */
export async function getSpec(id: string): Promise<TechnicalSpecDto> {
  return api.get<TechnicalSpecDto>(
    `/technical-specs/${encodeURIComponent(id)}`,
  );
}

/**
 * Crée une nouvelle version (bump) d'une spec existante.
 *
 * Backend :
 *   - 201 → nouvelle spec Draft renvoyée. L'ancienne passe `Superseded`.
 *   - 400/422 → version <= précédente (refus monotonie).
 *   - 403 → user ≠ syndic / superadmin.
 *   - 404 → spec source non trouvée.
 *
 * Si MAJOR bump → l'UI doit avertir l'utilisateur AVANT (signatures invalidées,
 * cf. `isMajorBump` ci-dessus + modal confirm dans TechnicalSpecDetail).
 */
export async function bumpVersion(
  id: string,
  req: BumpTechnicalSpecRequest,
): Promise<TechnicalSpecDto> {
  return api.post<TechnicalSpecDto>(
    `/technical-specs/${encodeURIComponent(id)}/bump`,
    req,
  );
}

/**
 * Soumet une spec Draft → status passe à `PendingSignatures`.
 *
 * Backend :
 *   - 200 → DTO mis à jour.
 *   - 403 → user ≠ syndic / superadmin.
 *   - 404 → spec non trouvée.
 *   - 400 → spec déjà soumise / non-Draft.
 */
export async function submitForSignatures(
  id: string,
): Promise<TechnicalSpecDto> {
  return api.post<TechnicalSpecDto>(
    `/technical-specs/${encodeURIComponent(id)}/submit`,
    {},
  );
}

/**
 * Signe une spec (un user/role). Si toutes les signatures sont présentes,
 * le backend passe la spec en status `Approved`.
 *
 * Backend :
 *   - 201 → SignatureDto renvoyée.
 *   - 400 → spec pas en PendingSignatures.
 *   - 403 → rôle non autorisé / mandate manquant pour rôle mandataire.
 *   - 404 → spec non trouvée.
 *   - 409 → signature déjà présente pour (user, role).
 */
export async function signSpec(
  id: string,
  req: SignTechnicalSpecRequest,
): Promise<TechnicalSpecSignatureDto> {
  return api.post<TechnicalSpecSignatureDto>(
    `/technical-specs/${encodeURIComponent(id)}/signatures`,
    req,
  );
}

/**
 * Liste les signatures d'une spec — utilisé par TechnicalSpecDetail pour la
 * section "Signatures" (ordre chronologique).
 */
export async function listSignatures(
  id: string,
): Promise<TechnicalSpecSignatureDto[]> {
  return api.get<TechnicalSpecSignatureDto[]>(
    `/technical-specs/${encodeURIComponent(id)}/signatures`,
  );
}
