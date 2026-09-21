// Story 2.2 — Scope store (Svelte 5 runes).
//
// ADR-0012 (Navigation contextualisée) + ADR-0011 (Portefeuille).
//
// Source de vérité réactive du périmètre de travail courant (building / acp /
// portfolio sélectionnés). Consommé par :
// - `BuildingSelector.svelte` (mutation + lecture) — le filtre immeuble,
//   secondaire depuis la story #798.
// - `AcpSelector.svelte` (mutation + lecture, story #798) — le périmètre
//   PRINCIPAL : l'ACP est la personne morale (numéro BCE, compte, AG,
//   quotités), pas l'immeuble. Une ACP couvre 1..N immeubles (ACP principale
//   + secondaires, droit belge) : la relation n'est jamais 1:1.
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
  /**
   * Combien d'ACP l'utilisateur peut-il atteindre ? `null` tant qu'on ne l'a
   * pas demandé au serveur.
   *
   * Ce n'est pas une donnée d'affichage, c'est ce qui décide si le CHOIX doit
   * exister. La remise de design en fait une règle : « Affordance is
   * conditional — chevron and selector sheet appear only when the owner
   * belongs to more than one ACP. A single choice is not a menu. »
   */
  acpsDisponibles: number | null;
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
  acpsDisponibles: null,
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
/**
 * La clé sous laquelle le choix de l'utilisateur survit à une navigation.
 *
 * ── Pourquoi une mémoire est nécessaire, et l'URL ne suffit pas ───────────
 *
 * Le frontend est une application Astro MULTI-PAGE : chaque clic de menu est
 * un chargement de document complet, et un `$state` de module repart à zéro.
 * Un lien profond `?buildingId=` répond au cas « j'arrive par une URL » ; il
 * ne répond pas au cas « je clique Dépenses », puisque ce lien ne porte
 * aucune chaîne de requête.
 *
 * Le repli serveur ne comble pas le trou non plus : il n'adopte une ACP que
 * si l'utilisateur en a **exactement une** (`resoudreLeDefautServeur`). Un
 * syndic multi-ACP — la prémisse du produit — repartait donc sans périmètre
 * à CHAQUE navigation, et devait re-sélectionner après chaque bouton.
 *
 * ── Pourquoi cela ne rouvre pas le risque que l'en-tête de ce module écarte ─
 *
 * L'en-tête refuse `localStorage` parce que persister le périmètre « crée un
 * risque de scope violation post-rotation d'organisation ». Le raisonnement
 * vaut, et il est respecté de deux façons :
 *
 *   1. **`sessionStorage`, pas `localStorage`** : la mémoire meurt avec
 *      l'onglet. Elle ne traverse ni un redémarrage du navigateur, ni une
 *      autre session.
 *   2. **Rien n'est cru sur parole.** L'identifiant mémorisé n'est adopté
 *      qu'après confirmation par la liste des ACP que le SERVEUR accorde à
 *      l'appelant. Une ACP qui n'y figure plus — rotation d'organisation,
 *      mandat clos, droit retiré — est ignorée et la mémoire effacée.
 *
 * C'est la même garantie que pour le lien profond : l'identifiant sert à
 * DEMANDER, jamais à affirmer.
 */
const MEMOIRE_PERIMETRE = "koprogo_perimetre_acp";

/**
 * La seconde moitié du périmètre — et pourquoi elle est arrivée après.
 *
 * #841 a posé la mémoire d'ACP. Le PO avait signalé les deux d'un seul
 * geste : « quand on sélectionne une ACP, dès qu'on appuie sur un bouton ou
 * qu'on va dans un menu il faut resélectionner ». La barre de contexte porte
 * DEUX sélecteurs, et seul le premier avait reçu sa mémoire.
 *
 * `selectedBuildingId` repartait donc à `null` à chaque chargement de
 * document — c'est-à-dire à chaque clic de menu, puisque le frontend est une
 * application Astro multi-page. Douze écrans lisent ce périmètre ;
 * `/journal-entries` s'ouvrait sur « choisissez un immeuble » à chaque visite,
 * et un comptable qui saisit dix écritures sur le même immeuble le
 * resélectionnait dix fois (#981).
 */
const MEMOIRE_IMMEUBLE = "koprogo_perimetre_immeuble";

function ecrire(cle: string, valeur: string | null): void {
  if (typeof window === "undefined") return;
  try {
    if (valeur === null) window.sessionStorage.removeItem(cle);
    else window.sessionStorage.setItem(cle, valeur);
  } catch {
    // Un navigateur qui refuse le stockage de session ne doit pas casser la
    // sélection : on perd la mémoire, pas la fonctionnalité.
  }
}

function lire(cle: string): string | null {
  if (typeof window === "undefined") return null;
  try {
    return window.sessionStorage.getItem(cle);
  } catch {
    return null;
  }
}

function memoriser(acpId: string | null): void {
  ecrire(MEMOIRE_PERIMETRE, acpId);
  // Une ACP qui tombe emporte l'immeuble : un immeuble mémorisé sans ACP
  // décrirait un périmètre à moitié posé, ce qui est pire que pas de
  // périmètre du tout — l'écran afficherait des données d'un immeuble sans
  // dire de quelle copropriété il relève.
  if (acpId === null) ecrire(MEMOIRE_IMMEUBLE, null);
}

function memorise(): string | null {
  return lire(MEMOIRE_PERIMETRE);
}

function memoriserLImmeuble(buildingId: string | null): void {
  ecrire(MEMOIRE_IMMEUBLE, buildingId);
}

function immeubleMemorise(): string | null {
  return lire(MEMOIRE_IMMEUBLE);
}

export function setBuilding(building: Building | null): void {
  if (building === null) {
    _state.selectedBuildingId = null;
    _state.selectedBuilding = null;
    // Désélectionner explicitement, c'est un choix : la mémoire doit le
    // suivre, sans quoi l'immeuble reviendrait au prochain écran.
    memoriserLImmeuble(null);
    return;
  }
  _state.selectedBuildingId = building.id;
  _state.selectedBuilding = building;
  _state.selectedAcpId = building.acp_id ?? null;
  _state.scopeError = null;
  memoriser(_state.selectedAcpId);
  memoriserLImmeuble(building.id);
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
  memoriser(acpId);
}

/**
 * Demande l'accès à une ACP donnée, VALIDÉ par le serveur (Story #798).
 *
 * `setAcp()` fait confiance à l'appelant — correct quand l'identifiant vient
 * déjà d'une liste scope-filtrée par le serveur (`listAcps()`). Ce chemin-ci
 * sert quand ce n'est pas garanti (lien profond, ACP demandée par un
 * identifiant externe) : le critère `@negative` de la story #798 exige qu'une
 * ACP hors du portefeuille de l'utilisateur retourne 403 **sans changer le
 * périmètre courant**.
 *
 * Différence avec `rehydraterDepuisLurl` (#841) : celui-là répond à un
 * premier chargement de page — le périmètre part de zéro, donc "reset" et
 * "inchangé" coïncident. Ici, un refus PENDANT une session ne doit rien
 * effacer de ce qui fonctionnait déjà : on ne touche `selectedAcpId` qu'en
 * cas de succès.
 *
 * @param acpId   Identifiant demandé (non cru sur parole).
 * @param charger Chargeur d'ACP par identifiant, injecté pour rester testable
 *                sans réseau et sans importer la couche API dans le store.
 * @returns       `true` si le serveur a confirmé l'ACP (scope mis à jour),
 *                `false` sinon (scope inchangé, `scopeError` posé).
 */
export async function demanderAcp(
  acpId: string,
  charger: (id: string) => Promise<{ id: string }>,
): Promise<boolean> {
  try {
    const acp = await charger(acpId);
    _state.selectedAcpId = acp.id;
    _state.scopeError = null;
    return true;
  } catch (err: unknown) {
    const statut = (err as { status?: number } | null)?.status;
    _state.scopeError = statut === 403 ? "forbidden" : "not_found";
    return false;
  }
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
    // Un immeuble que le serveur vient de refuser ne doit pas ressurgir au
    // chargement suivant : la mémoire périmée est la façon dont un refus se
    // transforme en boucle.
    memoriserLImmeuble(null);
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
 * Le périmètre par défaut, résolu par le serveur.
 *
 * ── Le défaut que cette fonction ferme ───────────────────────────────────
 *
 * Le frontend est une application Astro **multi-page** : chaque navigation
 * est un chargement de document complet, et un `$state` de module repart à
 * zéro. Le périmètre était donc **nul au premier rendu de chaque page**, sans
 * exception. Douze composants le lisent ; tous voyaient `null`.
 *
 * `JournalEntriesPanel` refuse d'afficher quoi que ce soit sans immeuble, et
 * ce refus est juste : une écriture comptable qui ne désigne pas sa
 * copropriété n'est imputable à personne. Ce qui n'allait pas, c'est
 * qu'**arriver sur la page par une URL ne permettait jamais de le
 * satisfaire** — il fallait cliquer le sélecteur pendant ce même chargement.
 * Recharger, revenir en arrière, ou ouvrir un signet ramenait l'écran vide.
 *
 * L'en-tête du store annonçait deux mécanismes de repli, un lien profond et
 * un défaut serveur. **Aucun des deux n'existait.** C'est l'issue #841.
 *
 * ── Pourquoi le défaut serveur d'abord, et le lien profond ensuite ───────
 *
 * Décision du 2026-09-10. Le défaut serveur couvre le cas courant sans
 * paramètre d'URL, et surtout **il ne peut pas boucler** : il ne lit rien de
 * ce que l'utilisateur contrôle. Un lien profond mal formé, lui, peut
 * relancer une résolution à chaque rendu — c'est ce qui avait cassé trois
 * recettes Playwright, avec un `networkidle` qui n'arrivait jamais.
 *
 * ── La règle, et ce qu'elle refuse de deviner ────────────────────────────
 *
 * Une seule ACP accessible ? C'est le périmètre. Plusieurs ? **On ne choisit
 * pas** : deviner ferait travailler un syndic dans la mauvaise copropriété
 * sans qu'il l'ait demandé, et les écritures qu'il y passerait seraient
 * imputées à la mauvaise personne morale. L'écran demande alors, et le
 * sélecteur affiche la liste **préchargée** — ce qui règle au passage le
 * défaut du sélecteur, dont l'état de repos était `isOpen = results.length >
 * 0` : cliquer le champ ne montrait rien tant qu'on n'avait pas tapé.
 *
 * @param charger  Chargeur de la liste des ACP accessibles. Injecté pour que
 *                 le store reste testable sans réseau.
 * @returns        L'identifiant adopté, ou `null` si le choix revient à
 *                 l'utilisateur — ou s'il n'a accès à aucune ACP.
 */
export async function resoudreLeDefautServeur(
  charger: () => Promise<{ id: string }[]>,
): Promise<string | null> {
  // Un périmètre déjà posé — par un lien profond, ou par un clic — n'est
  // jamais écrasé par le défaut. Le défaut comble une absence, il n'arbitre
  // pas.
  if (_state.selectedAcpId !== null) return _state.selectedAcpId;

  let acps: { id: string }[];
  try {
    acps = await charger();
  } catch {
    // Un défaut qu'on n'a pas pu résoudre n'est pas une erreur de périmètre :
    // l'utilisateur choisira. Poser `scopeError` afficherait un message de
    // refus là où il n'y a eu aucun refus.
    return null;
  }

  _state.acpsDisponibles = acps.length;
  if (acps.length !== 1) return null;

  _state.selectedAcpId = acps[0].id;
  _state.scopeError = null;
  return acps[0].id;
}

/** Chargeurs injectés pour {@link resoudrePerimetreAuChargement}. */
export interface ChargeursPerimetre {
  building: (id: string) => Promise<Building>;
  acps: () => Promise<{ id: string }[]>;
}

/**
 * Compose les deux replis en un seul point d'entrée pour le chargement d'une
 * page : c'est LA fonction qui manquait pour clore #841.
 *
 * `rehydraterDepuisLurl` et `resoudreLeDefautServeur` existaient déjà,
 * séparément testés, mais rien ne les enchaînait. Un appelant qui voulait les
 * trois branches de l'AC @edge — lien profond, puis défaut serveur, puis nul —
 * devait réinventer l'ordre et, surtout, la règle de non-repli ci-dessous.
 *
 * ── L'ordre ───────────────────────────────────────────────────────────────
 *
 * 1. `?buildingId=` (ou `building_id=`) accepté par le serveur → il gagne, le
 *    défaut n'est même pas consulté.
 * 2. Absence de paramètre → le défaut serveur tente de combler.
 * 3. Ni l'un ni l'autre → périmètre nul, l'écran demande une sélection.
 *
 * ── La règle qui n'est pas qu'un détail d'ordre ─────────────────────────────
 *
 * Un `?buildingId=` **présent mais refusé** par le serveur (403/404) ne
 * retombe PAS sur le défaut serveur. Deviner un remplacement à un lien que
 * l'utilisateur (ou l'attaquant) a explicitement demandé serait le repli
 * silencieux que l'AC @negative interdit — et pour un lien partagé entre deux
 * cabinets, cela ferait atterrir l'utilisateur sur SA PROPRE ACP par défaut
 * sans qu'aucun écran ne dise que le lien d'origine a été refusé.
 * `rehydraterDepuisLurl` a déjà posé `scopeError`, c'est cet état qui doit
 * rester visible.
 *
 * @param chargeurs  `building` sert le lien profond, `acps` sert le défaut.
 *                   Injectés pour que l'orchestrateur reste testable sans
 *                   réseau et n'importe pas la couche API.
 */
/**
 * Reprend le choix mémorisé pour la session, APRÈS confirmation du serveur.
 *
 * L'identifiant n'est jamais cru : il n'est adopté que s'il figure dans la
 * liste des ACP que le serveur accorde à l'appelant. S'il n'y figure plus —
 * rotation d'organisation, mandat clos, droit retiré — il est ignoré ET
 * effacé, pour qu'une mémoire périmée ne resurgisse pas au chargement
 * suivant.
 *
 * @returns L'ACP reprise, ou `null` si aucune mémoire ou si le serveur ne
 *          la reconnaît plus.
 */
export async function reprendreLeChoixDeLaSession(
  charger: () => Promise<{ id: string }[]>,
): Promise<string | null> {
  // Un périmètre déjà posé — par un lien profond, ou par un clic pendant ce
  // même chargement — n'a pas besoin d'être repris, et surtout ne doit pas
  // coûter une requête. Même garde que `resoudreLeDefautServeur` : le
  // souvenir comble une absence, il n'arbitre pas.
  //
  // Sans elle, un test existant tombait en disant exactement cela :
  // « un périmètre déjà posé avant l'appel n'est jamais écrasé par le
  // défaut » vérifiait aussi qu'AUCUNE requête ne partait.
  if (_state.selectedAcpId !== null) return _state.selectedAcpId;

  const souvenir = memorise();
  if (souvenir === null) return null;

  let acps: { id: string }[];
  try {
    acps = await charger();
  } catch {
    // Serveur injoignable : on ne tranche pas, et surtout on n'efface pas —
    // une panne de réseau ne doit pas faire oublier un choix légitime.
    return null;
  }

  _state.acpsDisponibles = acps.length;
  if (!acps.some((a) => a.id === souvenir)) {
    memoriser(null);
    return null;
  }

  _state.selectedAcpId = souvenir;
  _state.scopeError = null;
  return souvenir;
}

/**
 * Reprend l'immeuble choisi plus tôt dans la session — sans le croire.
 *
 * Même contrat que `reprendreLeChoixDeLaSession` pour l'ACP, plus une
 * exigence que l'ACP n'a pas : la COHÉRENCE. Restaurer un immeuble qui
 * relève d'une autre ACP que celle en cours donnerait un périmètre
 * contradictoire — un écran qui affiche les dépenses d'un immeuble sous
 * l'en-tête d'une autre copropriété. C'est pire qu'un périmètre vide, parce
 * que ça se lit comme une donnée juste.
 *
 * L'immeuble est donc demandé au SERVEUR, et adopté seulement si son
 * `acp_id` correspond au périmètre courant.
 *
 * @param charger Chargeur d'immeuble par identifiant, injecté pour rester
 *                testable sans réseau.
 */
export async function reprendreLImmeubleDeLaSession(
  charger: (id: string) => Promise<Building>,
): Promise<string | null> {
  // Un immeuble déjà posé — lien profond, ou clic pendant ce chargement —
  // n'a pas besoin d'être repris, et surtout ne doit pas coûter une requête.
  if (_state.selectedBuildingId !== null) return _state.selectedBuildingId;

  const souvenir = immeubleMemorise();
  if (souvenir === null) return null;

  let immeuble: Building;
  try {
    immeuble = await charger(souvenir);
  } catch {
    // Serveur injoignable, ou immeuble refusé. On ne tranche pas et on
    // n'efface pas : une panne de réseau ne doit pas faire oublier un choix
    // légitime. Le cas « le serveur ne le sert plus » est couvert par la
    // vérification de cohérence ci-dessous quand la requête aboutit, et par
    // `setScopeError` quand elle rend 403.
    return null;
  }

  // La cohérence. Un immeuble d'une autre ACP est ignoré, et oublié : le
  // garder ferait rejouer le même écart à chaque écran.
  const acpDeLImmeuble = immeuble.acp_id ?? null;
  if (
    _state.selectedAcpId !== null &&
    acpDeLImmeuble !== null &&
    acpDeLImmeuble !== _state.selectedAcpId
  ) {
    memoriserLImmeuble(null);
    return null;
  }

  _state.selectedBuildingId = immeuble.id;
  _state.selectedBuilding = immeuble;
  if (_state.selectedAcpId === null && acpDeLImmeuble !== null) {
    _state.selectedAcpId = acpDeLImmeuble;
  }
  return immeuble.id;
}

export async function resoudrePerimetreAuChargement(
  chargeurs: ChargeursPerimetre,
): Promise<void> {
  if (typeof window === "undefined") return;

  const params = new URLSearchParams(window.location.search);
  const idDemande = params.get("buildingId") ?? params.get("building_id");

  // L'ordre n'est pas arbitraire : ce que l'URL DEMANDE prime sur ce dont on
  // se souvient, et le souvenir prime sur un défaut choisi à la place de
  // l'utilisateur. Du plus explicite au moins explicite.
  const adopte = await rehydraterDepuisLurl(chargeurs.building);
  if (adopte !== null) return;
  if (idDemande !== null) return;

  const repris = await reprendreLeChoixDeLaSession(chargeurs.acps);
  if (repris === null) {
    await resoudreLeDefautServeur(chargeurs.acps);
  }

  // L'immeuble se reprend APRÈS l'ACP, et jamais avant : la vérification de
  // cohérence a besoin de savoir de quelle copropriété on parle. L'inverse
  // laisserait passer un immeuble d'une autre ACP le temps d'un rendu — et
  // un périmètre faux affiché une seconde est un périmètre faux.
  await reprendreLImmeubleDeLaSession(chargeurs.building);
}

/**
 * Reset complet du scope (logout, switch organization, fin de session).
 */
export function resetScope(): void {
  // La mémoire de session part avec le périmètre : une déconnexion ou une
  // rotation d'organisation ne doit rien laisser derrière elle.
  memoriser(null);
  memoriserLImmeuble(null);
  _state.selectedBuildingId = null;
  _state.selectedAcpId = null;
  _state.selectedPortfolioId = null;
  _state.selectedBuilding = null;
  _state.scopeError = null;
  _state.acpsDisponibles = null;
}

/**
 * Façade `scope` exposée aux composants `.svelte` — proxy `$state` sur le
 * state module-level. Lecture réactive via `scope.selectedBuildingId` dans
 * un composant runes (déclenche un rerender quand la valeur change).
 */
export const scope = _state;
