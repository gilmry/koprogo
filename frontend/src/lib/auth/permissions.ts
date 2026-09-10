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
 * Sous-ensemble du `ScopeSnapshot` que `canSee` consomme.
 *
 * ── Le périmètre bascule de l'immeuble vers l'ACP ───────────────────────
 *
 * `canSee` ne dépendait que de `selectedBuildingId`. C'est le mauvais porteur,
 * et l'ADR 0046 comme la remise de design le disent indépendamment l'une de
 * l'autre : **un syndic est mandaté par une ACP, jamais par un immeuble.** Une
 * ACP est une personne morale — numéro BCE, compte propre, assemblée, budget —
 * et elle peut compter plusieurs immeubles. La comptabilité y est attachée
 * (ADR-0045), les appels de fonds aussi.
 *
 * ── Pourquoi les DEUX pendant la transition ─────────────────────────────
 *
 * Le périmètre d'immeuble ne disparaît pas : il devient le **filtre
 * secondaire**, à l'intérieur de l'ACP. Il reste donc un porteur de contexte
 * légitime.
 *
 * Et il y a une raison de méthode : six répertoires de recettes Playwright et
 * trois fichiers de tests unitaires pilotent aujourd'hui `setBuilding()`.
 * Basculer d'un coup les casserait tous, et on ne saurait plus distinguer une
 * régression d'un ancrage à déplacer. La remise impose cette prudence :
 * « Only remove the building-scoped code path once every spec has an ACP
 * equivalent, in a separate commit. »
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
 * `contractor` y figure **par décision, pas par oubli** (2026-09-07, #815). Un
 * prestataire est un tiers extérieur à la copropriété : il intervient sur un
 * ticket, dépose un rapport, et repart. La voie nominale est le **lien
 * magique**, pas le compte — `pages/c.astro` et `pages/contractor/`, toutes
 * deux `requireAuth={false}`. Lui ouvrir un menu de navigation reviendrait à
 * le faire entrer dans le périmètre de l'ACP, ce qu'il ne doit jamais voir.
 *
 * Un compte portant ce rôle ne verra donc aucun menu, et c'est correct. Ce
 * qu'il verra, c'est le message de `ROLES_SANS_INTERFACE` — pas une barre
 * vide.
 *
 * `board_member` n'y figure plus depuis le 2026-09-07 : l'arbitrage de #816 est
 * rendu, et le conseil voit désormais trois menus. Voir `BOARD_MENUS`.
 *
 * Cette liste est le **registre des rôles sans interface** : `garde-roles`
 * exige que chaque rôle du backend voie au moins un menu ou figure ici. Un rôle
 * ajouté côté serveur et oublié côté interface fait alors échouer un test, au
 * lieu d'offrir un écran vide à un utilisateur.
 */
export const ROLES_SANS_INTERFACE: ReadonlySet<string> = new Set([
  "contractor",
  "lawyer",
  "notary",
  "amo",
  "architect",
  "bet",
  "warden",
]);

/**
 * Ce que voit un membre du conseil de copropriété.
 *
 * ── Ce qui fonde ce choix ──────────────────────────────────────────────────
 *
 * **Il est copropriétaire avant d'être conseiller.** L'Art. 3.90 § 1er réserve
 * le conseil aux « titulaires d'un droit réel disposant du droit de vote »,
 * et `conseil_de_copropriete.rs:66` l'implémente. Lui retirer « Mes lots » et
 * « Communauté » en le nommant au conseil serait lui faire perdre des droits
 * en gagnant une charge.
 *
 * **Sa mission est le contrôle du syndic.** L'Art. 3.90 § 2 lui ouvre les
 * pièces et documents se rapportant à la gestion — d'où `gouvernance`, qui
 * porte assemblées, convocations, décisions et documents légaux.
 *
 * ── Ce qu'il ne voit pas, et pourquoi ──────────────────────────────────────
 *
 * Ni `gestion` ni `compta` ni `ticketing`. Ce sont les menus de l'exécution
 * courante, celle qu'il surveille sans la conduire — lui en donner l'entrée
 * inviterait à confondre contrôle et cogestion, ce que la loi distingue.
 *
 * ── Ce que cette liste ne décide pas ───────────────────────────────────────
 *
 * `canSee` gouverne la VISIBILITÉ d'un menu, jamais le droit d'écrire. Les
 * gardes de route s'en chargent, et elles sont correctes :
 * `get_board_dashboard` vérifie `has_active_board_mandate(owner_id,
 * building_id)` avant de servir quoi que ce soit. Ouvrir un menu n'ouvre donc
 * aucune écriture.
 *
 * Décision du 2026-09-07, #816.
 */
const BOARD_MENUS: ReadonlySet<Menu> = new Set([
  "mes-lots",
  "communaute",
  "gouvernance",
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

  // « En contexte » = l'utilisateur travaille DANS une copropriété donnée.
  // L'ACP est le porteur ; l'immeuble reste accepté le temps que les recettes
  // migrent, et parce qu'il désigne toujours une ACP unique par son
  // rattachement (`setBuilding` renseigne `selectedAcpId` au passage).
  const hasBuildingScope =
    scope?.selectedAcpId != null || scope?.selectedBuildingId != null;

  // 2. Menu admin (gestion plateforme).
  if (menu === "admin") {
    if (!ADMIN_ROLES.has(role)) return false;
    // Mode in-context : on cache le menu admin si un building est sélectionné
    // pour éviter le mélange "menus business + menus admin" qui désoriente.
    return !hasBuildingScope;
  }

  // 3. Menu mes-lots (portail copropriétaire).
  //
  // `board_member` y figure parce qu'un membre du conseil EST copropriétaire
  // (Art. 3.90 § 1er) : la charge s'ajoute à sa qualité, elle ne la remplace
  // pas.
  if (menu === "mes-lots") {
    return (
      role === "owner" ||
      role === "community.moderator" ||
      role === "board_member"
    );
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

  // 4f. Membre du conseil de copropriété (#816).
  if (role === "board_member") return BOARD_MENUS.has(menu);

  // 4g. Rôle sans interface (cf. ROLES_SANS_INTERFACE) ou string inconnue →
  //     fail-closed. Story 3.4 introduira le scope mandat.
  return false;
}
