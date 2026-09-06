// Story 2.4 — permissions.canSee Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD : tests rouges AVANT le helper.
//
// Couverture matrice rôles × menus × scope :
// - @happy    : rôles "business" voient leurs menus quand un building est sélectionné
// - @edge     : admin/superadmin → mode "in-context" (menus business si selected,
//               menus admin/* sinon)
// - @security : accountant n'a JAMAIS de menu communauté, owner JAMAIS de gestion
// - @negative : role null / scope null / menu invalide → false strict
//
// Pourquoi un helper pur TypeScript (et pas inliner la logique dans Navigation) :
// - testabilité unitaire (zéro mount Svelte)
// - réutilisable par RouteGuard, BreadCrumbs, ContextMenu, etc.
// - i18n-safe (raisonne sur enum/key, jamais sur libellé traduit)

import { describe, it, expect } from "vitest";
import { canSee, type Menu, type Scope } from "../permissions";

// ---------------------------------------------------------------------------
// Helpers fixtures
// ---------------------------------------------------------------------------

const SCOPE_WITH_BUILDING: Scope = {
  selectedBuildingId: "11111111-1111-1111-1111-111111111111",
  selectedAcpId: "acp-001",
  selectedPortfolioId: null,
};

const SCOPE_EMPTY: Scope = {
  selectedBuildingId: null,
  selectedAcpId: null,
  selectedPortfolioId: null,
};

const BUSINESS_MENUS: Menu[] = [
  "gestion",
  "compta",
  "gouvernance",
  "communaute",
  "ticketing",
];

// ===========================================================================
// @happy — chemin nominal : chaque rôle voit ses menus quand scope est défini
// ===========================================================================

describe("canSee @happy", () => {
  it("syndic voit les 5 menus business quand un building est sélectionné", () => {
    for (const menu of BUSINESS_MENUS) {
      expect(canSee("syndic", menu, SCOPE_WITH_BUILDING)).toBe(true);
    }
  });

  it("owner voit communaute + mes-lots, rien d'autre", () => {
    expect(canSee("owner", "communaute", SCOPE_WITH_BUILDING)).toBe(true);
    expect(canSee("owner", "mes-lots", SCOPE_WITH_BUILDING)).toBe(true);
    // Pas de menus business pour l'owner
    expect(canSee("owner", "gestion", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "compta", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "gouvernance", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "ticketing", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("accountant voit compta uniquement (pas communaute, pas gestion)", () => {
    expect(canSee("accountant", "compta", SCOPE_WITH_BUILDING)).toBe(true);
    // Aucun autre menu business
    expect(canSee("accountant", "gestion", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("accountant", "gouvernance", SCOPE_WITH_BUILDING)).toBe(
      false,
    );
    expect(canSee("accountant", "communaute", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("accountant", "ticketing", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("accountant", "mes-lots", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("syndic ne voit pas mes-lots (réservé aux owners)", () => {
    expect(canSee("syndic", "mes-lots", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("syndic ne voit pas le menu admin", () => {
    expect(canSee("syndic", "admin", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("syndic", "admin", SCOPE_EMPTY)).toBe(false);
  });
});

// ===========================================================================
// @edge — admin/superadmin mode in-context
// ===========================================================================

describe("canSee @edge admin in-context", () => {
  it("admin SANS building sélectionné → voit menu admin uniquement", () => {
    expect(canSee("admin", "admin", SCOPE_EMPTY)).toBe(true);
    expect(canSee("admin", "admin", null)).toBe(true);
    // Pas de menus business si pas de scope
    for (const menu of BUSINESS_MENUS) {
      expect(canSee("admin", menu, SCOPE_EMPTY)).toBe(false);
      expect(canSee("admin", menu, null)).toBe(false);
    }
  });

  it("admin AVEC building sélectionné → voit menus business + masque admin", () => {
    for (const menu of BUSINESS_MENUS) {
      expect(canSee("admin", menu, SCOPE_WITH_BUILDING)).toBe(true);
    }
    // Menu admin masqué quand in-context (évite la confusion / mélange)
    expect(canSee("admin", "admin", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("superadmin a le même comportement in-context que admin", () => {
    expect(canSee("superadmin", "admin", SCOPE_EMPTY)).toBe(true);
    expect(canSee("superadmin", "admin", SCOPE_WITH_BUILDING)).toBe(false);
    for (const menu of BUSINESS_MENUS) {
      expect(canSee("superadmin", menu, SCOPE_WITH_BUILDING)).toBe(true);
      expect(canSee("superadmin", menu, SCOPE_EMPTY)).toBe(false);
    }
  });

  it("admin ne voit pas mes-lots (réservé aux owners, hors mode in-context)", () => {
    expect(canSee("admin", "mes-lots", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("superadmin", "mes-lots", SCOPE_WITH_BUILDING)).toBe(false);
  });
});

// ===========================================================================
// @security — RBAC strict, jamais d'escalade par rôle inférieur
// ===========================================================================

describe("canSee @security", () => {
  it("accountant ne voit JAMAIS le menu communaute (interdit produit)", () => {
    expect(canSee("accountant", "communaute", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("accountant", "communaute", SCOPE_EMPTY)).toBe(false);
    expect(canSee("accountant", "communaute", null)).toBe(false);
  });

  it("owner ne voit JAMAIS gestion/compta/gouvernance/ticketing", () => {
    expect(canSee("owner", "gestion", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "compta", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "gouvernance", SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("owner", "ticketing", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("owner ne voit JAMAIS le menu admin", () => {
    expect(canSee("owner", "admin", SCOPE_EMPTY)).toBe(false);
    expect(canSee("owner", "admin", SCOPE_WITH_BUILDING)).toBe(false);
  });

  it("aucun rôle (null) → aucun menu visible (fail-closed)", () => {
    for (const menu of [
      ...BUSINESS_MENUS,
      "mes-lots" as Menu,
      "admin" as Menu,
    ]) {
      expect(canSee(null, menu, SCOPE_WITH_BUILDING)).toBe(false);
      expect(canSee(null, menu, SCOPE_EMPTY)).toBe(false);
      expect(canSee(null, menu, null)).toBe(false);
    }
  });

  it("rôle inconnu/string aléatoire → aucun menu visible (fail-closed)", () => {
    for (const menu of BUSINESS_MENUS) {
      expect(canSee("hacker", menu, SCOPE_WITH_BUILDING)).toBe(false);
      expect(canSee("", menu, SCOPE_WITH_BUILDING)).toBe(false);
    }
  });

  it("community.moderator est traité comme un owner pour le menu communaute", () => {
    // Le nom vient du backend : `UserRole::CommunityModerator` s'affiche
    // `community.moderator`, avec un POINT (`user.rs:59`). Ce test employait
    // `community-moderator`, avec un trait d'union — la même faute que le
    // code qu'il gardait. Il passait donc en validant une constante qui
    // n'existe nulle part ailleurs, pendant qu'un vrai modérateur recevait
    // une navigation vide (#814).
    expect(
      canSee("community.moderator", "communaute", SCOPE_WITH_BUILDING),
    ).toBe(true);
    expect(canSee("community.moderator", "gestion", SCOPE_WITH_BUILDING)).toBe(
      false,
    );
    expect(canSee("community.moderator", "mes-lots", SCOPE_WITH_BUILDING)).toBe(
      true,
    );
  });

  it("les deux sous-rôles comptables voient compta, comme le comptable générique", () => {
    // `accountant.encodeur` (saisie amont) et `accountant.emetteur` (sortie
    // financière) sont servis par le backend depuis la story 3.1. Seul
    // `accountant` était reconnu : les deux autres tombaient en fail-closed
    // et recevaient une barre de navigation vide (#814).
    for (const role of ["accountant.encodeur", "accountant.emetteur"]) {
      expect(canSee(role, "compta", SCOPE_WITH_BUILDING)).toBe(true);
      expect(canSee(role, "gestion", SCOPE_WITH_BUILDING)).toBe(false);
      expect(canSee(role, "admin", null)).toBe(false);
    }
  });
});

// ===========================================================================
// @negative — entrées invalides, défaillances correctes (pas de throw)
// ===========================================================================

describe("canSee @negative", () => {
  it("ne throw jamais — toute entrée invalide retourne false", () => {
    expect(() => canSee(null, "gestion", null)).not.toThrow();
    expect(() => canSee("syndic", "unknown-menu" as Menu, null)).not.toThrow();
    expect(() => canSee(undefined as any, "gestion", null)).not.toThrow();
    expect(() =>
      canSee("syndic", "gestion", { selectedBuildingId: null } as any),
    ).not.toThrow();
  });

  it("scope partiel (objets dégénérés) → ne fait pas planter ; syndic toujours OK", () => {
    const degenerate = { selectedBuildingId: null } as Scope;
    // Syndic n'est pas conditionné par le scope (voir matrice — un syndic gère
    // toujours un building, mais le menu reste visible même avant sélection
    // pour qu'il puisse naviguer vers le BuildingSelector).
    expect(canSee("syndic", "gestion", degenerate)).toBe(true);
    // Admin SANS building → menu admin
    expect(canSee("admin", "admin", degenerate)).toBe(true);
    // Admin SANS building → pas de menus business
    expect(canSee("admin", "gestion", degenerate)).toBe(false);
  });

  it("menu inconnu → false (pas de fallback permissif)", () => {
    expect(canSee("syndic", "xyz" as Menu, SCOPE_WITH_BUILDING)).toBe(false);
    expect(canSee("admin", "xyz" as Menu, SCOPE_EMPTY)).toBe(false);
  });
});
