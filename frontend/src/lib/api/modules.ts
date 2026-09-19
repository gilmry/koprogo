// Client API des modules ACP (activation/désactivation par capacité).
//
// ADR-0015 (modularité par capacité). Story 5.1 (#585, backend) apporte la
// table `acp_enabled_modules`, l'entité domaine et l'endpoint ci-dessous.
//
// ── La dette écrite en tête de ce fichier est SOLDÉE (2026-09-16) ─────────
//
// Cette Story 5.2 (#586, UI) avait été écrite avant que #585 n'existe, et
// tenait donc `ModuleName` et le DTO de réponse à la main, faute de types
// générés. #585 étant livrée et ses handlers enregistrés dans
// `infrastructure/openapi.rs`, les deux viennent maintenant de `api.d.ts`,
// c'est-à-dire de la source Rust.
//
// `MODULE_NAMES` reste écrit ici, parce qu'un type TypeScript s'efface à la
// compilation et qu'`isKnownModule` a besoin d'une liste à l'exécution. Mais
// il ne peut plus diverger en silence : `VERIFICATION_EXHAUSTIVITE` ci-dessous
// ne compile que si la liste couvre exactement l'énumération Rust.
//
// Endpoints (architecture.md §6.1) :
//   GET /acps/{id}/modules                   → { acp_id, modules }
//   PUT /acps/{id}/modules/{module}/enable   → 204
//   PUT /acps/{id}/modules/{module}/disable  → 204

import type { components } from "../../types/api";

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
] as const satisfies readonly ModuleName[];

/**
 * Empêche `MODULE_NAMES` d'OUBLIER une variante.
 *
 * `satisfies` ci-dessus interdit d'y mettre un nom qui n'existe pas côté
 * Rust ; il n'interdit pas d'en omettre un. Cette constante ferme l'autre
 * sens : le type est `never` dès qu'une variante de `ModuleName` n'apparaît
 * pas dans la liste, et le fichier cesse alors de compiler.
 *
 * C'est l'équivalent TypeScript de `garde_enum_contre_contrainte` côté Rust,
 * qui tient l'énumération et la contrainte SQL alignées dans les deux sens.
 */
type VariantesOubliees = Exclude<ModuleName, (typeof MODULE_NAMES)[number]>;
const _VERIFICATION_EXHAUSTIVITE: VariantesOubliees extends never
  ? true
  : never = true;
void _VERIFICATION_EXHAUSTIVITE;

/** Nom de module, repris de la source Rust via `api.d.ts` (enum `Module`). */
export type ModuleName = components["schemas"]["Module"];

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

/** Réponse de `GET /acps/{id}/modules`, reprise de la source Rust. */
export type EnabledModulesResponseDto =
  components["schemas"]["EnabledModulesResponseDto"];

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
