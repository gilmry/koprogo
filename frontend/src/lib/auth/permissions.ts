// Story 2.4 — permissions.ts (helper RBAC menus contextualisés).
//
// ADR-0012 (Navigation contextualisée) — menu visibility = f(role, scope).
//
// Pourquoi un module pur TypeScript (pas une store Svelte) :
// - Testable en Vitest sans monter de composant.
// - Importable depuis Navigation.svelte ET depuis RouteGuard, BreadCrumbs,
//   ContextMenu, etc. Pas de dépendance circulaire.
// - i18n-safe : on raisonne sur des clés ('gestion', 'compta', ...) jamais sur
//   des libellés traduits ('Gestion', 'Beheer'). Cf. memory data-testid-systematic.
//
// Pourquoi pas inliner dans Navigation.svelte :
// - Logique permission `if role === 'syndic' || role === 'admin'` répétée à
//   chaque menu = nightmare de maintenance.
// - Impossible à tester sans render du composant (slow + flaky).
// - Cf. CRITICAL §4 typed errors / single source of truth.
//
// Évolution prévue (story 3.1) :
// - Sub-rôles `accountant.encodeur` / `accountant.emetteur` raffineront `compta`.
// - Sub-rôle `community.moderator` aura des actions modération en plus.
// - `lawyer` / `notary` / `amo` / `architect` / `bet` : scope mandat (story 3.4).

/**
 * Clés de menus stables (i18n-safe — jamais traduit, jamais affiché brut à
 * l'utilisateur ; sert uniquement de pivot pour `canSee()` et `data-testid`).
 *
 * - `gestion`     : owners, units, expenses, contributions...
 * - `compta`      : invoices, journal-entries, reports PCMN...
 * - `gouvernance` : meetings, convocations, board, documents légaux...
 * - `communaute`  : SEL, polls, notices, bookings, sharing, skills...
 * - `ticketing`   : tickets, quotes, work-reports, inspections...
 * - `mes-lots`    : portail propriétaire (units, payments, profil)
 * - `admin`       : menus plateforme (`/admin/*`) — réservé super/cabinet admins
 */
export type Menu =
  | "gestion"
  | "compta"
  | "gouvernance"
  | "communaute"
  | "ticketing"
  | "mes-lots"
  | "admin";

/**
 * Type rôle élargi (string accepté car BE peut renvoyer un sub-rôle pas encore
 * mappé côté FE — cf. story 3.1). `null` = utilisateur non authentifié ou sans
 * UserRoleAssignment actif.
 */
export type Role =
  | "superadmin"
  | "admin"
  | "syndic"
  | "accountant"
  | "accountant.encodeur"
  | "accountant.emetteur"
  | "owner"
  | "community.moderator"
  | string
  | null;

/**
 * Subset minimal du `ScopeSnapshot` consommé par `canSee`. On ne dépend QUE
 * de `selectedBuildingId` ici — les autres champs (acpId, portfolioId) sont
 * orthogonaux au menu visibility. Une story future pourra étendre si
 * `selectedPortfolioId` débloque des menus spécifiques.
 */
export type Scope = {
  selectedBuildingId: string | null;
  selectedAcpId?: string | null;
  selectedPortfolioId?: string | null;
} | null;

/**
 * Rôles qui peuvent voir le menu admin (gestion plateforme).
 * Coïncide avec ceux qui basculent en mode "in-context".
 */
const ADMIN_ROLES: ReadonlySet<string> = new Set(["superadmin", "admin"]);

/**
 * Rôles qui voient les 5 menus business quand un building est sélectionné.
 * (Syndic toujours ; admin/superadmin uniquement en mode in-context.)
 */
const BUSINESS_ROLES_ALWAYS: ReadonlySet<string> = new Set(["syndic"]);

/**
 * Les rôles comptables, générique et sous-rôles.
 *
 * `UserRole` distingue depuis la story 3.1 la saisie amont
 * (`accountant.encodeur` — facture, devis) de la sortie financière
 * (`accountant.emetteur` — charges, appels de fonds). Les deux sous-rôles
 * travaillent dans le même menu ; ce qu'ils peuvent y faire relève des gardes
 * de route, pas de la visibilité du menu.
 *
 * Avant le 2026-09-06, seul `accountant` était reconnu : un comptable encodeur
 * tombait en fail-closed et recevait une navigation entièrement vide (#814).
 */
const ACCOUNTING_ROLES: ReadonlySet<string> = new Set([
  "accountant",
  "accountant.encodeur",
  "accountant.emetteur",
]);

/**
 * Rôles servis par le backend qui n'ont **délibérément** aucune interface.
 *
 * Le fail-closed est correct pour eux : aucun écran ne leur est destiné. Ce qui
 * ne l'est pas, c'est de le leur montrer sous la forme d'une barre de
 * navigation vide — c'est le sujet de #814.
 *
 * `contractor` et `board_member` figurent ici **à titre provisoire**. Leurs
 * écrans existent (`pages/contractor/`, `pages/board-dashboard.astro`,
 * `BoardDashboard.svelte`, `DecisionTracker.svelte`) et leurs routes serveur
 * aussi — dix pour `/board-members`, neuf pour `/board-decisions`, quinze pour
 * `/contractor-reports`. Ce qui manque est une décision, pas du code : quels
 * menus, dans quel périmètre. Un prestataire est un tiers qui n'a rien à voir
 * du dossier d'ACP ; un membre du conseil surveille le syndic (Art. 3.90 § 1er)
 * et a donc besoin de lecture large sans écriture. Voir #815 et #816.
 *
 * Cette liste est le **registre des rôles sans interface** : `garde-roles`
 * exige que chaque rôle du backend voie au moins un menu ou figure ici. Un rôle
 * ajouté côté serveur et oublié côté interface fait alors échouer un test, au
 * lieu d'offrir un écran vide à un utilisateur.
 */
export const ROLES_SANS_INTERFACE: ReadonlySet<string> = new Set([
  "contractor",
  "board_member",
  "lawyer",
  "notary",
  "amo",
  "architect",
  "bet",
  "warden",
]);

/**
 * Détermine si un menu doit être visible pour un rôle dans un scope donné.
 *
 * Règles :
 * 1. **Pas de rôle** → tout false (fail-closed).
 * 2. **Menu admin** :
 *    - admin/superadmin SANS building → true (mode plateforme).
 *    - admin/superadmin AVEC building → false (mode in-context : on cache le
 *      menu admin pour éviter la confusion).
 *    - autres rôles → false toujours.
 * 3. **Menus business (gestion/compta/gouvernance/communaute/ticketing)** :
 *    - syndic → toujours visibles (un syndic gère un building précis ; pas
 *      de mode multi-tenant à ce niveau).
 *    - admin/superadmin → visibles UNIQUEMENT si building sélectionné
 *      (mode in-context).
 *    - accountant → uniquement `compta`.
 *    - owner → uniquement `communaute` (+ `mes-lots`).
 *    - community.moderator → comme owner pour `communaute`.
 * 4. **Menu mes-lots** :
 *    - owner / community.moderator → true.
 *    - autres → false (les pros n'ont pas de "Mes lots").
 *
 * @param role  Rôle actif de l'utilisateur (UserRoleAssignment.role)
 * @param menu  Clé de menu (cf. type Menu)
 * @param scope Scope courant (cf. stores/scope.svelte.ts)
 * @returns true si le menu doit être rendu, false sinon (fail-closed)
 */
export function canSee(role: Role, menu: Menu, scope: Scope): boolean {
  // 1. Rôle null/undefined/vide → fail-closed.
  if (!role || typeof role !== "string") return false;

  const hasBuildingScope = scope?.selectedBuildingId != null;

  // 2. Menu admin (gestion plateforme).
  if (menu === "admin") {
    if (!ADMIN_ROLES.has(role)) return false;
    // Mode in-context : on cache le menu admin si un building est sélectionné
    // pour éviter le mélange "menus business + menus admin" qui désoriente.
    return !hasBuildingScope;
  }

  // 3. Menu mes-lots (portail copropriétaire).
  if (menu === "mes-lots") {
    return role === "owner" || role === "community.moderator";
  }

  // 4. Menus business (gestion/compta/gouvernance/communaute/ticketing).
  const isBusinessMenu =
    menu === "gestion" ||
    menu === "compta" ||
    menu === "gouvernance" ||
    menu === "communaute" ||
    menu === "ticketing";

  if (!isBusinessMenu) {
    // Menu inconnu — fail-closed (pas de fallback permissif).
    return false;
  }

  // 4a. Syndic → tous les menus business toujours.
  if (BUSINESS_ROLES_ALWAYS.has(role)) return true;

  // 4b. Admin/Superadmin en mode in-context (building sélectionné).
  if (ADMIN_ROLES.has(role)) return hasBuildingScope;

  // 4c. Comptable, générique ou sous-rôle → uniquement compta.
  if (ACCOUNTING_ROLES.has(role)) return menu === "compta";

  // 4d. Owner → uniquement communaute.
  if (role === "owner") return menu === "communaute";

  // 4e. Modérateur communauté (sub-rôle, story 3.1) → communaute uniquement.
  //
  // Le backend sérialise `community.moderator`, avec un POINT
  // (`user.rs:59`). L'interface comparait à `community-moderator`, avec un
  // trait d'union : la comparaison ne pouvait jamais réussir, et le
  // modérateur recevait une navigation vide. Le test unitaire ne le voyait
  // pas — il employait la même constante fautive que le code (#814).
  if (role === "community.moderator") return menu === "communaute";

  // 4f. Rôle sans interface (cf. ROLES_SANS_INTERFACE) ou string inconnue →
  //     fail-closed. Story 3.4 introduira le scope mandat.
  return false;
}
