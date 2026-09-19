// Story 5.2 — Store `enabled_modules` (Svelte 5 runes).
//
// ADR-0015 (modularité par capacité). Cache par ACP des modules activés,
// tenu à jour par `ModuleGate.svelte` au fil des bascules de `scope`
// (cf. `stores/scope.svelte.ts`, champ `selectedAcpId`).
//
// Même pattern que `scope.svelte.ts` (cf. son en-tête) : PAS de classe avec
// des champs `$state` exportée directement — ce pattern échoue en mode
// production Vitest, le rune n'étant pas évalué hors `.svelte`. On utilise
// donc un objet `$state` privé au module + des fonctions mutateurs pures
// exportées, plus une façade `enabledModules` pour la lecture réactive.
//
// Fail-closed volontaire : un ACP dont le fetch échoue est mémorisé avec un
// ensemble de modules VIDE (rien n'est affiché) plutôt que laissé "non
// chargé" indéfiniment — cela évite à la fois une boucle de re-fetch et un
// affichage optimiste de contenu que le serveur n'a pas confirmé (cf. AC
// @security de la Story 5.2 : le gate ne décide jamais tout seul).

import { SvelteMap } from "svelte/reactivity";
import { listEnabledModules, type ModuleName } from "../lib/api/modules";

export interface EnabledModulesSnapshot {
  /** Modules activés, par ACP déjà résolue (avec succès ou en échec fail-closed). */
  byAcp: SvelteMap<string, Set<ModuleName>>;
  /** ACP dont le dernier fetch a échoué — diagnostic, pas un blocage de retry définitif. */
  erroredAcpId: string | null;
}

// `byAcp` est un `SvelteMap`, pas un `Map` brut : un `Map`/`Set` imbriqué
// dans `$state` n'est PAS rendu profondément réactif par Svelte 5 (seuls
// objets/tableaux le sont) — ses mutations via `.set()`/`.delete()`
// resteraient invisibles aux lecteurs réactifs. Même pattern que
// `JournalEntryList.svelte` / `PollDetail.svelte` dans ce repo.
const _state = $state<EnabledModulesSnapshot>({
  byAcp: new SvelteMap(),
  erroredAcpId: null,
});

/**
 * Façade exposée aux composants `.svelte` — lecture réactive via
 * `enabledModules.byAcp` dans un composant runes.
 */
export const enabledModules = _state;

/** Les modules de `acpId` ont-ils déjà été résolus (succès ou échec) ? */
export function isAcpModulesLoaded(acpId: string): boolean {
  return _state.byAcp.has(acpId);
}

/** `module` est-il activé pour `acpId` ? `false` si non résolu ou acpId nul. */
export function isModuleEnabled(
  acpId: string | null,
  module: ModuleName,
): boolean {
  if (!acpId) return false;
  return _state.byAcp.get(acpId)?.has(module) ?? false;
}

/**
 * Charge (ou recharge) les modules activés d'une ACP depuis le serveur.
 * Injecté avec `listEnabledModules` réel — pas de paramètre d'injection ici
 * car, contrairement à `scope.svelte.ts`, ce store a une seule source
 * réseau, déjà mockable au niveau du module dans les tests (cf.
 * `ModuleGate.test.ts`).
 */
export async function loadEnabledModulesForAcp(acpId: string): Promise<void> {
  try {
    const modules = await listEnabledModules(acpId);
    _state.byAcp.set(acpId, new Set(modules));
    _state.erroredAcpId = null;
  } catch {
    // Fail-closed : ACP marquée comme résolue, mais sans aucun module actif.
    _state.byAcp.set(acpId, new Set());
    _state.erroredAcpId = acpId;
  }
}

/** Reset complet (logout, fin de session, isolation entre tests). */
export function resetEnabledModules(): void {
  _state.byAcp = new SvelteMap();
  _state.erroredAcpId = null;
}
