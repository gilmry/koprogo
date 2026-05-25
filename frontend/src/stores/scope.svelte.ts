// Story 2.2 — Scope store (Svelte 5 runes).
//
// ADR-0012 (Navigation contextualisée) + ADR-0011 (Portefeuille).
//
// Source de vérité réactive du périmètre de travail courant (building / acp /
// portfolio sélectionnés). Consommé par :
// - `BuildingSelector.svelte` (mutation + lecture)
// - `ContextBanner.svelte` (Story 2.3 — affichage)
// - menus contextualisés (Navigation) — masquer/afficher liens selon scope
//
// Pourquoi runes et pas `writable()` (cf. CLAUDE.md tech stack Svelte 5) :
// - Svelte 5 `$state` / `$derived` sont la voie canonique ; `writable()` reste
//   pour compat legacy uniquement (auth.ts, toast.ts, notifications.ts).
// - Aucun get() boilerplate ; consommation directe `scope.selectedBuildingId`.
//
// IMPORTANT — pas de persistance localStorage :
// - Le scope est dérivable d'un deep-link (?buildingId=...) ou d'un défaut
//   serveur (favori du portefeuille de l'utilisateur).
// - Persister le scope crée un risque de scope violation post-rotation
//   d'organisation (cf. AC @security : building hors scope → 403 + reset).
// - Le rehydrate sur reload sera porté par Story 2.5 (deep-links).

import type { Building } from "../lib/types";

/**
 * Snapshot scope minimal — partagé entre composants pour rerender.
 *
 * Note : on n'utilise PAS le pattern « class avec $state fields » directement
 * exporté car il échoue avec vitest mode production (le rune n'est pas évalué
 * hors `.svelte`). Pattern retenu : objets `$state` créés au niveau module +
 * fonctions mutateurs pures. Les composants `.svelte` peuvent dériver via
 * `$derived(scope.selectedBuildingId)` côté usage.
 */

export interface ScopeSnapshot {
  selectedBuildingId: string | null;
  selectedAcpId: string | null;
  selectedPortfolioId: string | null;
  selectedBuilding: Building | null;
  /**
   * État d'erreur scope — set quand un building cliqué retourne 403 (hors
   * scope du user) ou que le périmètre est invalide. L'UI affiche le testid
   * `building-selector-403` quand `scopeError === 'forbidden'`.
   */
  scopeError: null | "forbidden" | "not_found";
}

/**
 * Module-level scope state (singleton, Svelte 5 runes proxy).
 *
 * `$state` wraps l'objet dans un Proxy qui :
 * 1. notifie les composants lecteurs lors d'une mutation de champ,
 * 2. fonctionne en `.svelte.ts` (suffixe obligatoire pour le compileur runes).
 *
 * Usage côté composant `.svelte` :
 *   ```svelte
 *   <script>
 *     import { scope, setBuilding } from "../stores/scope.svelte";
 *   </script>
 *   <p>Building courant : {scope.selectedBuildingId ?? "aucun"}</p>
 *   <button onclick={() => setBuilding(null)}>Reset</button>
 *   ```
 *
 * Les tests Vitest snap-shot via `getScope()` (clone non-réactif).
 */
const _state = $state<ScopeSnapshot>({
  selectedBuildingId: null,
  selectedAcpId: null,
  selectedPortfolioId: null,
  selectedBuilding: null,
  scopeError: null,
});

/**
 * Lecture courante du scope (snapshot non réactif — pour assertions tests
 * ou code impératif comme un API call).
 */
export function getScope(): ScopeSnapshot {
  return { ..._state };
}

/**
 * Sélectionne un building (et son ACP parent si disponible).
 * Reset l'erreur scope si elle était présente.
 */
export function setBuilding(building: Building | null): void {
  if (building === null) {
    _state.selectedBuildingId = null;
    _state.selectedBuilding = null;
    return;
  }
  _state.selectedBuildingId = building.id;
  _state.selectedBuilding = building;
  // Acp inféré si exposé sur le DTO ; Story 1.2 renomme organization_id → acp_id
  // mais le frontend type Building expose encore `organization_id` (legacy).
  // On lit prudemment les deux pour rester compatible le temps du rebranding FE.
  const acpId =
    (building as Building & { acp_id?: string }).acp_id ??
    building.organization_id ??
    null;
  _state.selectedAcpId = acpId;
  _state.scopeError = null;
}

/**
 * Sélectionne un portfolio (peut coexister avec un building s'il appartient
 * au portfolio — handled par la couche UI, pas ici).
 */
export function setPortfolio(portfolioId: string | null): void {
  _state.selectedPortfolioId = portfolioId;
}

/**
 * Sélectionne directement un ACP (override le ACP dérivé d'un building).
 * Utilisé par les pages ACP-niveau (Story 1.x ACP listing).
 */
export function setAcp(acpId: string | null): void {
  _state.selectedAcpId = acpId;
}

/**
 * Signale un échec de scope (403 backend ou not_found).
 *
 * Le composant `BuildingSelector` peut alors :
 * 1. Afficher le testid `building-selector-403`.
 * 2. Reset la sélection (pour ne pas laisser une UI incohérente).
 */
export function setScopeError(
  error: null | "forbidden" | "not_found",
): void {
  _state.scopeError = error;
  if (error !== null) {
    // Reset selection — évite que le banner Story 2.3 affiche un building
    // que l'utilisateur ne peut pas voir.
    _state.selectedBuildingId = null;
    _state.selectedBuilding = null;
  }
}

/**
 * Reset complet du scope (logout, switch organization, fin de session).
 */
export function resetScope(): void {
  _state.selectedBuildingId = null;
  _state.selectedAcpId = null;
  _state.selectedPortfolioId = null;
  _state.selectedBuilding = null;
  _state.scopeError = null;
}

/**
 * Façade `scope` exposée aux composants `.svelte` — proxy `$state` sur le
 * state module-level. Lecture réactive via `scope.selectedBuildingId` dans
 * un composant runes (déclenche un rerender quand la valeur change).
 */
export const scope = _state;
