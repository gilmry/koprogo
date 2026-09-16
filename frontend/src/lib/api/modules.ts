// Client API des modules ACP (activation/désactivation par capacité).
//
// ADR-0015 (modularité par capacité). Story 5.1 (#585, backend) introduit la
// table `acp_enabled_modules`, l'entité domaine, le middleware `ModuleGuard`
// et l'endpoint ci-dessous — **mais n'est pas encore mergée** au moment où
// cette Story 5.2 (#586, UI) est écrite. Ni migration, ni handler, ni schéma
// OpenAPI n'existent pour l'instant.
//
// Cette dette est volontaire et documentée (comme `AcpAvecMetriques` dans
// `acps.ts` pour une raison différente) : `ModuleName` et le DTO de réponse
// sont donc définis ici à la main plutôt qu'importés de `types/api.d.ts`.
// À remplacer par les types générés dès que Story 5.1 enregistre son handler
// dans `infrastructure/openapi.rs`.
//
// Endpoint attendu (architecture.md §6.1) :
//   GET /acps/{id}/modules → { acp_id, modules: ModuleName[] }
//   (scope guard — visible seulement pour les utilisateurs de cette ACP)

import { api } from "../api";

/**
 * Noms de modules connus (ADR-0015). `identity` est toujours actif côté
 * backend (Story 5.1 @negative : tentative de désactivation refusée), mais
 * reste une valeur valide de `module` ici — ModuleGate ne fait que refléter
 * ce que le serveur répond, jamais de règle d'activation câblée côté client.
 */
export const MODULE_NAMES = [
  "identity",
  "community",
  "ticketing",
  "accounting",
  "governance",
  "maintenance",
  "portfolio",
] as const;

export type ModuleName = (typeof MODULE_NAMES)[number];

/** Vérifie qu'un nom de module (typiquement une prop `string`) est connu. */
export function isKnownModule(name: string): name is ModuleName {
  return (MODULE_NAMES as readonly string[]).includes(name);
}

/**
 * Levée (et seulement journalisée en console, jamais affichée à
 * l'utilisateur) quand `<ModuleGate module="...">` reçoit un nom de module
 * qui n'existe pas dans `MODULE_NAMES`. Cf. AC @negative de la Story 5.2 :
 * échouer visiblement côté développeur, silencieusement côté utilisateur
 * (fragment vide, pas de message d'erreur affiché).
 */
export class UnknownModuleError extends Error {
  constructor(readonly moduleName: string) {
    super(`ModuleGate: module inconnu "${moduleName}"`);
    this.name = "UnknownModuleError";
  }
}

/** DTO de réponse temporaire — cf. dette documentée en tête de fichier. */
export interface EnabledModulesResponseDto {
  acp_id: string;
  modules: ModuleName[];
}

/**
 * Liste les modules activés pour une ACP donnée.
 *
 * `silent: true` : un échec (403 ModuleGuard, endpoint pas encore déployé
 * pendant la transition Story 5.1) est une dégradation attendue, pas une
 * erreur utilisateur — `enabled_modules.svelte.ts` le traite en fail-closed
 * (aucun module affiché) sans toast intrusif.
 */
export async function listEnabledModules(acpId: string): Promise<ModuleName[]> {
  const dto = await api.get<EnabledModulesResponseDto>(
    `/acps/${encodeURIComponent(acpId)}/modules`,
    { silent: true },
  );
  return dto.modules;
}
