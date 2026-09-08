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
  _state.selectedAcpId = building.acp_id ?? null;
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
export function setScopeError(error: null | "forbidden" | "not_found"): void {
  _state.scopeError = error;
  if (error !== null) {
    // Reset selection — évite que le banner Story 2.3 affiche un building
    // que l'utilisateur ne peut pas voir.
    _state.selectedBuildingId = null;
    _state.selectedBuilding = null;
  }
}

/**
 * Réhydrate le périmètre depuis l'URL, en le faisant VALIDER par le serveur.
 *
 * ── Le défaut que cela corrige ────────────────────────────────────────────
 *
 * Ce module dit lui-même, en tête, que le périmètre « est dérivable d'un
 * deep-link (?buildingId=...) ou d'un défaut serveur », et que « le rehydrate
 * sur reload sera porté par Story 2.5 ». **Aucun des deux n'existait.**
 *
 * Or le frontend est une application Astro MULTI-PAGE : chaque navigation est
 * un chargement de document complet, et un `$state` de module repart à zéro.
 * Le périmètre était donc nul au premier rendu de CHAQUE page, sans exception,
 * pour les douze composants qui le lisent.
 *
 * `/journal-entries` en est l'illustration : l'écran refuse à juste titre une
 * écriture sans immeuble — une pièce comptable qui ne désigne pas sa
 * copropriété n'est imputable à personne — mais **arriver par une URL ne
 * permettait jamais de satisfaire ce refus**. Il fallait cliquer le sélecteur
 * pendant ce même chargement ; recharger, revenir, ou ouvrir un signet
 * ramenait l'écran vide. Cf. #841.
 *
 * ── Pourquoi cela ne rouvre pas le risque que l'en-tête écarte ────────────
 *
 * L'en-tête refuse la persistance parce qu'elle « crée un risque de scope
 * violation post-rotation d'organisation ». Le raisonnement vaut, et il est
 * respecté ici : **l'identifiant lu dans l'URL n'est jamais cru sur parole.**
 * Il sert à demander l'immeuble au serveur, qui applique ses propres gardes de
 * périmètre. Un 403 ou un 404 laisse le périmètre nul et lève `scopeError` —
 * exactement le chemin qu'emprunte déjà une sélection refusée.
 *
 * Un lien partagé entre deux cabinets ne donne donc accès à rien.
 *
 * @param charger  Chargeur d'immeuble par identifiant. Injecté pour que le
 *                 store reste testable sans réseau, et pour qu'il n'importe
 *                 pas la couche API.
 * @returns        L'immeuble adopté, ou `null` si l'URL n'en désignait aucun
 *                 ou si le serveur l'a refusé.
 */
export async function rehydraterDepuisLurl(
  charger: (id: string) => Promise<Building>,
): Promise<Building | null> {
  if (typeof window === "undefined") return null;

  const params = new URLSearchParams(window.location.search);
  // `buildingId` est la forme canonique ; `building_id` existe déjà dans
  // `tickets.astro` et `tickets/new.astro`, et on l'accepte plutôt que de
  // casser des liens qui circulent peut-être déjà.
  const id = params.get("buildingId") ?? params.get("building_id");
  if (!id) return null;

  try {
    const building = await charger(id);
    setBuilding(building);
    return building;
  } catch (err: unknown) {
    // Le serveur a refusé : on ne garde RIEN. `setScopeError` remet la
    // sélection à zéro, ce qui évite d'afficher un immeuble que l'appelant
    // n'a pas le droit de voir.
    const statut = (err as { status?: number } | null)?.status;
    setScopeError(statut === 403 ? "forbidden" : "not_found");
    return null;
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
